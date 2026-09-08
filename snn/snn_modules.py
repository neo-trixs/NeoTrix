"""
NeoTrix SNN (Spiking Neural Network) - Comprehensive Implementation
====================================================================

Implements all core SNN modules for the NeoTrix consciousness architecture:

1. STDP (Spike-Timing-Dependent Plasticity) Learning Rule
2. Adaptive LIF Neurons (ALIF)
3. Time-to-Spike (T2S) Encoder
4. Spiking Transformer Layer (Attention in spike domain)
5. Spiking GPT-2 (Autoregressive language model with spiking neurons)
6. Spiking MLP (Multi-Layer Perceptron with spiking activation)
7. SNN-ViT (Spiking Vision Transformer)

All modules operate in discrete time steps, using binary spike events
(0 or 1) and membrane potential dynamics.

Architecture:
    Input → [T2S Encoder] → [SNN Layers] → [STDP Learning] → Output

License: MIT
Author: NeoTrix Agent
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import Optional, Tuple, List, Dict, Any

import torch
import torch.nn as nn
import torch.nn.functional as F
from torch import Tensor


# =============================================================================
# Configuration
# =============================================================================

@dataclass
class SNNConfig:
    """Global configuration for SNN modules."""
    neuron_type: str = "alif"  # "lif" or "alif"
    tau_mem: float = 20.0
    tau_adapt: float = 100.0
    threshold: float = 1.0
    v_rest: float = 0.0
    v_reset: float = -0.5
    adapt_increment: float = 0.5
    surrogate_slope: float = 5.0
    dt: float = 1.0
    timesteps: int = 15


@dataclass
class TransformerConfig:
    """Configuration for Spiking Transformer."""
    d_model: int = 256
    n_heads: int = 8
    n_layers: int = 6
    d_ff: int = 1024
    dropout: float = 0.1
    max_seq_len: int = 512
    vocab_size: int = 50257
    timesteps: int = 15


@dataclass
class ViTConfig:
    """Configuration for Spiking ViT."""
    image_size: int = 224
    patch_size: int = 16
    d_model: int = 384
    n_heads: int = 6
    n_layers: int = 12
    n_classes: int = 1000
    timesteps: int = 15


# =============================================================================
# Surrogate Gradient Functions
# =============================================================================

class SurrogateGradient(torch.autograd.Function):
    """
    Surrogate gradient for non-differentiable spike function.

    Forward: hard threshold (step function)
    Backward: smooth sigmoid surrogate gradient
    """

    @staticmethod
    def forward(ctx, x: Tensor, threshold: float, slope: float) -> Tensor:
        ctx.save_for_backward(x)
        ctx.threshold = threshold
        ctx.slope = slope
        return (x >= threshold).float()

    @staticmethod
    def backward(ctx, grad_output: Tensor) -> Tuple[Tensor, None, None]:
        x, = ctx.saved_tensors
        threshold = ctx.threshold
        slope = ctx.slope

        x_shifted = slope * (x - threshold)
        sig = torch.sigmoid(x_shifted)
        grad_input = grad_output * sig * (1 - sig) * slope

        return grad_input, None, None


def surrogate_sigmoid(x: Tensor, threshold: float = 1.0, slope: float = 5.0) -> Tensor:
    """Apply spike with surrogate gradient."""
    return SurrogateGradient.apply(x, threshold, slope)


# =============================================================================
# 1. Adaptive LIF Neuron
# =============================================================================

class AdaptiveLIFNeuron(nn.Module):
    """
    Adaptive Leaky Integrate-and-Fire (ALIF) Neuron.

    Dynamics:
        tau_m * dV/dt = -(V - V_rest) + I
        tau_adapt * dtheta/dt = -(theta - theta_0) + adapt_inc * spikes

    When V >= theta: neuron fires, V resets, theta increases.

    Adaptation implements:
    - Spike-frequency adaptation (neurons fire slower over time)
    - Attention-like gain control
    - Natural attention decay (aligns with GWT attention dynamics)
    """

    def __init__(
        self,
        n_neurons: int,
        config: Optional[SNNConfig] = None,
        **kwargs
    ):
        super().__init__()
        self.config = config or SNNConfig(**kwargs)
        self.n_neurons = n_neurons

        # Learnable parameters
        self.tau_mem = nn.Parameter(
            torch.full((n_neurons,), self.config.tau_mem)
        )
        self.tau_adapt = nn.Parameter(
            torch.full((n_neurons,), self.config.tau_adapt)
        )
        self.threshold_base = nn.Parameter(
            torch.full((n_neurons,), self.config.threshold)
        )

        self.v_rest = self.config.v_rest
        self.v_reset = self.config.v_reset
        self.adapt_increment = self.config.adapt_increment
        self.dt = self.config.dt

        # State buffers
        self.register_buffer('v_mem', torch.zeros(n_neurons))
        self.register_buffer('threshold', torch.ones(n_neurons) * self.config.threshold)
        self.register_buffer('spike_history', torch.zeros(n_neurons, 10))
        self.register_buffer('step_count', torch.tensor(0, dtype=torch.long))

    def reset_state(self, batch_size: Optional[int] = None):
        """Reset all neuron states."""
        if batch_size is not None:
            self.v_mem = torch.zeros(batch_size, self.n_neurons, device=self.v_mem.device)
            self.threshold = torch.ones(batch_size, self.n_neurons, device=self.threshold.device) * self.config.threshold
        else:
            self.v_mem = torch.zeros(self.n_neurons, device=self.v_mem.device)
            self.threshold = torch.ones(self.n_neurons, device=self.threshold.device) * self.config.threshold
        self.spike_history.zero_()
        self.step_count.zero_()

    def forward(self, input_current: Tensor) -> Tensor:
        """
        Forward pass for one time step.

        Args:
            input_current: [batch, n_neurons] synaptic input current

        Returns:
            spikes: [batch, n_neurons] binary spike tensor
        """
        batch_size = input_current.shape[0] if input_current.dim() > 1 else 1

        # Ensure state has batch dimension
        if self.v_mem.dim() == 1:
            self.v_mem = self.v_mem.unsqueeze(0).expand(batch_size, -1).clone()
            self.threshold = self.threshold.unsqueeze(0).expand(batch_size, -1).clone()

        # Membrane dynamics
        tau_m = F.softplus(self.tau_mem) + 1.0
        dv = (-(self.v_mem - self.v_rest) + input_current) * self.dt / tau_m
        self.v_mem = self.v_mem + dv

        # Spike detection with surrogate gradient
        spikes = surrogate_sigmoid(
            self.v_mem,
            threshold=self.threshold,
            slope=self.config.surrogate_slope
        )

        # Reset where spikes occurred
        self.v_mem = torch.where(
            spikes > 0,
            torch.full_like(self.v_mem, self.v_reset),
            self.v_mem
        )

        # Adaptation
        tau_a = F.softplus(self.tau_adapt) + 1.0
        dtheta = -(self.threshold - self.threshold_base) * self.dt / tau_a
        self.threshold = self.threshold + dtheta + self.adapt_increment * spikes

        # Update history
        self.spike_history = torch.roll(self.spike_history, 1, dims=-1)
        if spikes.dim() > 1:
            self.spike_history[..., 0] = spikes.mean(dim=0) if batch_size > 1 else spikes.squeeze(0)
        self.step_count += 1

        return spikes

    def extra_repr(self) -> str:
        return (
            f"n_neurons={self.n_neurons}, "
            f"tau_mem={self.config.tau_mem}, "
            f"threshold={self.config.threshold}"
        )


# =============================================================================
# 2. Spike Generator
# =============================================================================

class SpikeGenerator(nn.Module):
    """
    Binary spike generation with surrogate gradient for backprop.
    """

    def __init__(self, slope: float = 5.0):
        super().__init__()
        self.slope = slope

    def forward(self, membrane_potential: Tensor, threshold: float = 1.0) -> Tensor:
        """Generate spikes from membrane potential."""
        return surrogate_sigmoid(membrane_potential, threshold, self.slope)


# =============================================================================
# 3. Time-to-Spike Encoder
# =============================================================================

class TimeToSpikeEncoder(nn.Module):
    """
    Time-to-Spike (T2S) Encoding.

    Converts continuous values to spike trains:
        Value 1.0 -> spike at t=0 (earliest)
        Value 0.0 -> spike at t=T-1 (latest)
        Value v   -> spike at t = (1-v) * (T-1)

    Temporal coding advantages:
        - One spike per value: O(1) vs O(T) for rate coding
        - Early spikes = high priority (aligns with GWT)
    """

    def __init__(self, timesteps: int = 15, encoding: str = "temporal"):
        super().__init__()
        self.timesteps = timesteps
        self.encoding = encoding

        self.scale = nn.Parameter(torch.ones(1))
        self.bias = nn.Parameter(torch.zeros(1))

    def temporal_encode(self, x: Tensor) -> Tensor:
        """Temporal encoding: one spike per neuron at computed time."""
        batch, features = x.shape

        x_scaled = torch.sigmoid(x * self.scale + self.bias)
        spike_times = (1.0 - x_scaled) * (self.timesteps - 1)

        t_range = torch.arange(
            self.timesteps, device=x.device, dtype=x.dtype
        ).view(1, 1, -1)

        spike_times_exp = spike_times.unsqueeze(-1)
        spikes = (t_range >= spike_times_exp).float()

        return spikes

    def rate_encode(self, x: Tensor) -> Tensor:
        """Rate encoding: spike probability proportional to value."""
        x_scaled = torch.sigmoid(x * self.scale + self.bias)
        rand = torch.rand(
            *x_scaled.shape, self.timesteps, device=x.device
        )
        return (rand < x_scaled.unsqueeze(-1)).float()

    def forward(self, x: Tensor) -> Tensor:
        """Encode continuous values to spikes."""
        if self.encoding == "temporal":
            return self.temporal_encode(x)
        return self.rate_encode(x)

    def decode(self, spikes: Tensor) -> Tensor:
        """Decode spike train back to continuous values."""
        if self.encoding == "temporal":
            spike_times = torch.argmax(spikes, dim=-1).float()
            return 1.0 - spike_times / (self.timesteps - 1)
        return spikes.mean(dim=-1)

    def extra_repr(self) -> str:
        return f"timesteps={self.timesteps}, encoding={self.encoding}"


# =============================================================================
# 4. Spiking Attention
# =============================================================================

class SpikingAttention(nn.Module):
    """
    Spiking Self-Attention mechanism.

    Q, K, V projections -> LIF neurons -> spikes.
    Attention scores via temporal correlation of spike trains.
    """

    def __init__(
        self,
        d_model: int,
        n_heads: int,
        timesteps: int = 15,
        dropout: float = 0.1,
    ):
        super().__init__()
        self.d_model = d_model
        self.n_heads = n_heads
        self.timesteps = timesteps
        self.head_dim = d_model // n_heads

        assert d_model % n_heads == 0

        self.q_proj = nn.Linear(d_model, d_model)
        self.k_proj = nn.Linear(d_model, d_model)
        self.v_proj = nn.Linear(d_model, d_model)
        self.out_proj = nn.Linear(d_model, d_model)

        self.q_neuron = AdaptiveLIFNeuron(d_model, SNNConfig(timesteps=timesteps))
        self.k_neuron = AdaptiveLIFNeuron(d_model, SNNConfig(timesteps=timesteps))
        self.v_neuron = AdaptiveLIFNeuron(d_model, SNNConfig(timesteps=timesteps))

        self.dropout = nn.Dropout(dropout)
        self.scale = math.sqrt(self.head_dim)

    def forward(self, x: Tensor, mask: Optional[Tensor] = None) -> Tensor:
        """Forward pass through spiking attention."""
        batch, seq_len, _ = x.shape

        q = self.q_proj(x)
        k = self.k_proj(x)
        v = self.v_proj(x)

        # Flatten to 2D for neuron processing: [batch*seq_len, d_model]
        q_flat = q.view(-1, self.d_model)
        k_flat = k.view(-1, self.d_model)
        v_flat = v.view(-1, self.d_model)

        q_spikes = []
        k_spikes = []
        v_spikes = []

        flat_batch = batch * seq_len
        self.q_neuron.reset_state(flat_batch)
        self.k_neuron.reset_state(flat_batch)
        self.v_neuron.reset_state(flat_batch)

        for t in range(self.timesteps):
            q_spikes.append(self.q_neuron(q_flat))
            k_spikes.append(self.k_neuron(k_flat))
            v_spikes.append(self.v_neuron(v_flat))

        # Reshape back to [batch, seq_len, d_model, T]
        q_spikes = torch.stack(q_spikes, dim=-1).view(batch, seq_len, self.d_model, self.timesteps)
        k_spikes = torch.stack(k_spikes, dim=-1).view(batch, seq_len, self.d_model, self.timesteps)
        v_spikes = torch.stack(v_spikes, dim=-1).view(batch, seq_len, self.d_model, self.timesteps)

        attn_output = torch.zeros_like(x)

        for t in range(self.timesteps):
            q_t = q_spikes[..., t]
            k_t = k_spikes[..., t]
            v_t = v_spikes[..., t]

            q_t = q_t.view(batch, seq_len, self.n_heads, self.head_dim).transpose(1, 2)
            k_t = k_t.view(batch, seq_len, self.n_heads, self.head_dim).transpose(1, 2)
            v_t = v_t.view(batch, seq_len, self.n_heads, self.head_dim).transpose(1, 2)

            attn = torch.matmul(q_t, k_t.transpose(-2, -1)) / self.scale

            if mask is not None:
                attn = attn.masked_fill(
                    mask.unsqueeze(1).unsqueeze(2) == 0, float('-inf')
                )

            attn = F.softmax(attn, dim=-1)
            attn = self.dropout(attn)

            out_t = torch.matmul(attn, v_t)
            out_t = out_t.transpose(1, 2).contiguous().view(
                batch, seq_len, self.d_model
            )

            attn_output = attn_output + out_t

        attn_output = attn_output / self.timesteps

        return self.out_proj(attn_output)


# =============================================================================
# 5. Lateral Inhibition
# =============================================================================

class LateralInhibition(nn.Module):
    """
    Competitive lateral inhibition mechanism.

    Winner-take-all dynamics: only strongest neurons fire.
    """

    def __init__(self, n_neurons: int, strength: float = 0.5, top_k: int = 0):
        super().__init__()
        self.n_neurons = n_neurons
        self.strength = strength
        self.top_k = top_k

        self.inhibition_weights = nn.Parameter(
            torch.randn(n_neurons, n_neurons) * 0.1
        )

    def forward(self, spikes: Tensor) -> Tensor:
        """Apply lateral inhibition."""
        if self.top_k > 0 and self.top_k < self.n_neurons:
            values, indices = torch.topk(spikes, self.top_k, dim=-1)
            mask = torch.zeros_like(spikes)
            mask.scatter_(-1, indices, 1.0)
            return spikes * mask

        if spikes.dim() == 2:
            inhibition = F.linear(spikes, self.inhibition_weights)
            return spikes - self.strength * inhibition.clamp(min=0)
        else:
            batch, seq_len, n = spikes.shape
            spikes_flat = spikes.view(-1, n)
            inhibition = F.linear(spikes_flat, self.inhibition_weights)
            inhibited = spikes_flat - self.strength * inhibition.clamp(min=0)
            return inhibited.view(batch, seq_len, n)


# =============================================================================
# 6. Spiking FFN
# =============================================================================

class SpikingFFN(nn.Module):
    """
    Spiking Feed-Forward Network with lateral inhibition.
    """

    def __init__(
        self,
        d_model: int,
        d_ff: int,
        timesteps: int = 15,
        dropout: float = 0.1,
        inhibition_strength: float = 0.5,
    ):
        super().__init__()
        self.d_model = d_model
        self.d_ff = d_ff
        self.timesteps = timesteps

        self.linear1 = nn.Linear(d_model, d_ff)
        self.linear2 = nn.Linear(d_ff, d_model)

        self.neuron = AdaptiveLIFNeuron(d_ff, SNNConfig(timesteps=timesteps))
        self.inhibition = LateralInhibition(d_ff, strength=inhibition_strength)

        self.dropout = nn.Dropout(dropout)
        self.norm = nn.LayerNorm(d_model)

    def forward(self, x: Tensor) -> Tensor:
        """Forward pass through spiking FFN."""
        residual = x
        batch, seq_len, d_model = x.shape

        h = self.linear1(x)  # [batch, seq_len, d_ff]

        # Flatten for neuron: [batch*seq_len, d_ff]
        h_flat = h.view(-1, self.d_ff)
        flat_batch = batch * seq_len
        self.neuron.reset_state(flat_batch)

        spike_output = torch.zeros_like(h_flat)
        for t in range(self.timesteps):
            spikes = self.neuron(h_flat)
            spikes = self.inhibition(spikes)
            spike_output = spike_output + spikes

        spike_output = (spike_output / self.timesteps).view(batch, seq_len, self.d_ff)

        out = self.linear2(spike_output)
        out = self.dropout(out)

        return self.norm(residual + out)


# =============================================================================
# 7. Spiking Transformer Layer
# =============================================================================

class SpikingTransformerLayer(nn.Module):
    """
    Single Spiking Transformer Layer.

    Input -> [Spiking Attention] -> Add & Norm
          -> [Spiking FFN] -> Add & Norm -> Output
    """

    def __init__(
        self,
        d_model: int = 256,
        n_heads: int = 8,
        d_ff: int = 1024,
        timesteps: int = 15,
        dropout: float = 0.1,
    ):
        super().__init__()
        self.d_model = d_model
        self.timesteps = timesteps

        self.attention = SpikingAttention(
            d_model=d_model,
            n_heads=n_heads,
            timesteps=timesteps,
            dropout=dropout,
        )
        self.norm1 = nn.LayerNorm(d_model)
        self.dropout1 = nn.Dropout(dropout)

        self.ffn = SpikingFFN(
            d_model=d_model,
            d_ff=d_ff,
            timesteps=timesteps,
            dropout=dropout,
        )

    def forward(self, x: Tensor, mask: Optional[Tensor] = None) -> Tensor:
        """Forward pass."""
        attn_out = self.attention(x, mask=mask)
        x = self.norm1(x + self.dropout1(attn_out))
        x = self.ffn(x)
        return x


# =============================================================================
# 8. Spiking GPT-2
# =============================================================================

class SpikingGPT2(nn.Module):
    """
    Spiking GPT-2: Autoregressive language model with spiking neurons.

    Architecture:
        Token/Position Embedding -> [Spiking Transformer x N] -> Head
    """

    def __init__(
        self,
        config: Optional[TransformerConfig] = None,
        **kwargs
    ):
        super().__init__()
        self.config = config or TransformerConfig(**kwargs)

        self.token_emb = nn.Embedding(self.config.vocab_size, self.config.d_model)
        self.pos_emb = nn.Embedding(self.config.max_seq_len, self.config.d_model)

        self.layers = nn.ModuleList([
            SpikingTransformerLayer(
                d_model=self.config.d_model,
                n_heads=self.config.n_heads,
                d_ff=self.config.d_ff,
                timesteps=self.config.timesteps,
                dropout=self.config.dropout,
            )
            for _ in range(self.config.n_layers)
        ])

        self.final_norm = nn.LayerNorm(self.config.d_model)
        self.head = nn.Linear(self.config.d_model, self.config.vocab_size, bias=False)
        self.head.weight = self.token_emb.weight

        self.apply(self._init_weights)
        print(f"Spiking GPT-2: {self.get_num_params()/1e6:.1f}M parameters")

    def _init_weights(self, module):
        if isinstance(module, nn.Linear):
            torch.nn.init.normal_(module.weight, mean=0.0, std=0.02)
            if module.bias is not None:
                torch.nn.init.zeros_(module.bias)
        elif isinstance(module, nn.Embedding):
            torch.nn.init.normal_(module.weight, mean=0.0, std=0.02)
        elif isinstance(module, nn.LayerNorm):
            torch.nn.init.ones_(module.weight)
            torch.nn.init.zeros_(module.bias)

    def get_num_params(self, non_embedding: bool = True) -> int:
        n_params = sum(p.numel() for p in self.parameters())
        if non_embedding:
            n_params -= self.pos_emb.weight.numel()
        return n_params

    def forward(
        self,
        idx: Tensor,
        targets: Optional[Tensor] = None,
        mask: Optional[Tensor] = None,
    ) -> Tuple[Tensor, Optional[Tensor]]:
        """Forward pass."""
        batch, seq_len = idx.shape

        tok_emb = self.token_emb(idx)
        pos = torch.arange(0, seq_len, dtype=torch.long, device=idx.device)
        pos_emb = self.pos_emb(pos).unsqueeze(0)

        x = tok_emb + pos_emb

        for layer in self.layers:
            x = layer(x, mask=mask)

        x = self.final_norm(x)
        logits = self.head(x)

        loss = None
        if targets is not None:
            loss = F.cross_entropy(
                logits.view(-1, logits.size(-1)),
                targets.view(-1),
                ignore_index=-1,
            )

        return logits, loss

    @torch.no_grad()
    def generate(
        self,
        idx: Tensor,
        max_new_tokens: int = 100,
        temperature: float = 0.8,
        top_k: Optional[int] = 50,
    ) -> Tensor:
        """Autoregressive generation."""
        for _ in range(max_new_tokens):
            idx_crop = idx if idx.size(1) <= self.config.max_seq_len else \
                       idx[:, -self.config.max_seq_len:]

            logits, _ = self(idx_crop)
            logits = logits[:, -1, :] / temperature

            if top_k is not None:
                v, _ = torch.topk(logits, min(top_k, logits.size(-1)))
                logits[logits < v[:, [-1]]] = float('-inf')

            probs = F.softmax(logits, dim=-1)
            idx_next = torch.multinomial(probs, num_samples=1)
            idx = torch.cat((idx, idx_next), dim=1)

        return idx


# =============================================================================
# 9. Spiking MLP
# =============================================================================

class SpikingMLP(nn.Module):
    """
    Spiking Multi-Layer Perceptron.
    """

    def __init__(
        self,
        input_dim: int,
        hidden_dims: List[int],
        output_dim: int,
        timesteps: int = 15,
        dropout: float = 0.1,
    ):
        super().__init__()
        self.timesteps = timesteps

        dims = [input_dim] + hidden_dims + [output_dim]
        self.linear_layers = nn.ModuleList()
        self.neurons = nn.ModuleList()
        self.norms = nn.ModuleList()

        for i in range(len(dims) - 1):
            self.linear_layers.append(nn.Linear(dims[i], dims[i + 1]))
            if i < len(dims) - 2:
                self.neurons.append(
                    AdaptiveLIFNeuron(dims[i + 1], SNNConfig(timesteps=timesteps))
                )
                self.norms.append(nn.LayerNorm(dims[i + 1]))

        self.dropout = nn.Dropout(dropout)
        self.encoder = TimeToSpikeEncoder(timesteps=timesteps)
        self.apply(self._init_weights)

    def _init_weights(self, module):
        if isinstance(module, nn.Linear):
            torch.nn.init.kaiming_normal_(module.weight, nonlinearity='relu')
            if module.bias is not None:
                torch.nn.init.zeros_(module.bias)

    def forward(
        self,
        x: Tensor,
        return_spikes: bool = False,
    ) -> Tensor:
        """Forward pass."""
        batch = x.shape[0]
        all_spikes = {}

        current = x
        for i, (linear, neuron, norm) in enumerate(
            zip(self.linear_layers, self.neurons, self.norms)
        ):
            current = linear(current)

            layer_spikes = []
            neuron.reset_state(batch)

            for t in range(self.timesteps):
                spike = neuron(current)
                layer_spikes.append(spike)

            current = torch.stack(layer_spikes, dim=-1).sum(dim=-1)
            current = norm(current)
            current = self.dropout(current)

            if return_spikes:
                all_spikes[f'layer_{i}'] = torch.stack(layer_spikes, dim=-1)

        output = self.linear_layers[-1](current)

        if return_spikes:
            return output, all_spikes
        return output


# =============================================================================
# 10. SNN-ViT
# =============================================================================

class PatchEmbedding(nn.Module):
    """Convert image to patch embeddings."""

    def __init__(
        self,
        image_size: int = 224,
        patch_size: int = 16,
        in_channels: int = 3,
        d_model: int = 384,
    ):
        super().__init__()
        self.n_patches = (image_size // patch_size) ** 2
        self.projection = nn.Conv2d(
            in_channels, d_model,
            kernel_size=patch_size,
            stride=patch_size
        )

    def forward(self, x: Tensor) -> Tensor:
        """[B, C, H, W] -> [B, n_patches, d_model]"""
        x = self.projection(x)
        x = x.flatten(2).transpose(1, 2)
        return x


class SpikingViT(nn.Module):
    """
    Spiking Vision Transformer.
    """

    def __init__(
        self,
        config: Optional[ViTConfig] = None,
        in_channels: int = 3,
        **kwargs
    ):
        super().__init__()
        self.config = config or ViTConfig(**kwargs)

        self.patch_embed = PatchEmbedding(
            image_size=self.config.image_size,
            patch_size=self.config.patch_size,
            in_channels=in_channels,
            d_model=self.config.d_model,
        )
        self.n_patches = self.patch_embed.n_patches

        self.cls_token = nn.Parameter(
            torch.randn(1, 1, self.config.d_model) * 0.02
        )
        self.pos_embed = nn.Parameter(
            torch.randn(1, self.n_patches + 1, self.config.d_model) * 0.02
        )
        self.pos_drop = nn.Dropout(0.1)

        self.layers = nn.ModuleList([
            SpikingTransformerLayer(
                d_model=self.config.d_model,
                n_heads=self.config.n_heads,
                d_ff=self.config.d_model * 4,
                timesteps=self.config.timesteps,
                dropout=0.1,
            )
            for _ in range(self.config.n_layers)
        ])

        self.norm = nn.LayerNorm(self.config.d_model)
        self.head = nn.Linear(self.config.d_model, self.config.n_classes)

        nn.init.trunc_normal_(self.pos_embed, std=0.02)
        self.apply(self._init_weights)

        print(f"SNN-ViT: {sum(p.numel() for p in self.parameters())/1e6:.1f}M params")

    def _init_weights(self, module):
        if isinstance(module, nn.Linear):
            torch.nn.init.trunc_normal_(module.weight, std=0.02)
            if module.bias is not None:
                torch.nn.init.zeros_(module.bias)
        elif isinstance(module, nn.LayerNorm):
            torch.nn.init.ones_(module.weight)
            torch.nn.init.zeros_(module.bias)

    def forward(
        self,
        x: Tensor,
        return_all_tokens: bool = False,
    ) -> Tensor:
        """Forward pass."""
        batch = x.shape[0]

        x = self.patch_embed(x)
        cls_tokens = self.cls_token.expand(batch, -1, -1)
        x = torch.cat([cls_tokens, x], dim=1)
        x = x + self.pos_embed
        x = self.pos_drop(x)

        for layer in self.layers:
            x = layer(x)

        x = self.norm(x)
        cls_output = x[:, 0]
        logits = self.head(cls_output)

        if return_all_tokens:
            return logits, x
        return logits


# =============================================================================
# 11. STDP Learning Rule
# =============================================================================

class STDPLearner(nn.Module):
    """
    Three-factor STDP Learning Rule.

    delta_W = eligibility_trace * reward_signal
    """

    def __init__(
        self,
        n_pre: int,
        n_post: int,
        a_plus: float = 0.01,
        a_minus: float = 0.012,
        tau_plus: float = 20.0,
        tau_minus: float = 20.0,
        w_max: float = 1.0,
        w_min: float = 0.0,
        eligibility_decay: float = 0.95,
        reward_modulated: bool = True,
    ):
        super().__init__()
        self.n_pre = n_pre
        self.n_post = n_post

        self.weights = nn.Parameter(torch.randn(n_pre, n_post) * 0.1)

        self.register_buffer('eligibility', torch.zeros(n_pre, n_post))
        self.register_buffer('pre_trace', torch.zeros(n_pre))
        self.register_buffer('post_trace', torch.zeros(n_post))

        self.a_plus = a_plus
        self.a_minus = a_minus
        self.tau_plus = tau_plus
        self.tau_minus = tau_minus
        self.w_max = w_max
        self.w_min = w_min
        self.eligibility_decay = eligibility_decay
        self.reward_modulated = reward_modulated

    def reset_state(self):
        """Reset learning state."""
        self.eligibility.zero_()
        self.pre_trace.zero_()
        self.post_trace.zero_()

    def forward(
        self,
        pre_spikes: Tensor,
        post_spikes: Tensor,
        reward: Optional[Tensor] = None,
    ) -> Tensor:
        """Apply STDP learning."""
        self.pre_trace = self.pre_trace + (
            -self.pre_trace + pre_spikes
        ) / self.tau_plus
        self.post_trace = self.post_trace + (
            -self.post_trace + post_spikes
        ) / self.tau_minus

        # Standard STDP eligibility trace computation:
        # LTP: when post fires, strengthen based on recent pre activity (pre_trace)
        # LTD: when pre fires, weaken based on recent post activity (post_trace)
        eligibility = (
            self.a_plus * torch.outer(self.pre_trace, post_spikes)
            - self.a_minus * torch.outer(pre_spikes, self.post_trace)
        )

        self.eligibility = self.eligibility * self.eligibility_decay + eligibility

        if reward is not None and self.reward_modulated:
            weight_update = self.eligibility * reward
        else:
            weight_update = self.eligibility

        weight_update = torch.clamp(weight_update, -0.1, 0.1)

        self.weights.data = self.weights.data + weight_update
        self.weights.data = torch.clamp(self.weights.data, self.w_min, self.w_max)

        return weight_update

    def forward_inference(self, pre_spikes: Tensor) -> Tensor:
        """Forward pass for inference (no learning)."""
        return F.linear(pre_spikes, self.weights.T)

    def get_weight_stats(self) -> Dict[str, float]:
        """Weight statistics."""
        return {
            'mean': self.weights.mean().item(),
            'std': self.weights.std().item(),
            'max': self.weights.max().item(),
            'min': self.weights.min().item(),
        }


# =============================================================================
# 12. Model Factory
# =============================================================================

def create_snn_model(model_type: str = "spiking_mlp", **kwargs) -> nn.Module:
    """Factory function to create SNN models."""
    if model_type == "spiking_mlp":
        return SpikingMLP(
            input_dim=kwargs.get('input_dim', 784),
            hidden_dims=kwargs.get('hidden_dims', [256, 128]),
            output_dim=kwargs.get('output_dim', 10),
            timesteps=kwargs.get('timesteps', 15),
        )
    elif model_type == "spiking_gpt2":
        return SpikingGPT2(
            d_model=kwargs.get('d_model', 256),
            n_heads=kwargs.get('n_heads', 8),
            n_layers=kwargs.get('n_layers', 6),
            d_ff=kwargs.get('d_ff', 1024),
            vocab_size=kwargs.get('vocab_size', 50257),
            timesteps=kwargs.get('timesteps', 15),
        )
    elif model_type == "snn_vit":
        return SpikingViT(
            image_size=kwargs.get('image_size', 224),
            patch_size=kwargs.get('patch_size', 16),
            d_model=kwargs.get('d_model', 384),
            n_heads=kwargs.get('n_heads', 6),
            n_layers=kwargs.get('n_layers', 12),
            n_classes=kwargs.get('n_classes', 1000),
            timesteps=kwargs.get('timesteps', 15),
        )
    else:
        raise ValueError(f"Unknown model type: {model_type}")


# =============================================================================
# Main: Example Usage
# =============================================================================

if __name__ == "__main__":
    print("=" * 70)
    print("NeoTrix SNN - Module Test")
    print("=" * 70)

    # Test 1: Adaptive LIF Neuron
    print("\n--- Test 1: Adaptive LIF Neuron ---")
    neuron = AdaptiveLIFNeuron(n_neurons=100)
    input_current = torch.randn(32, 100) * 0.5

    spikes_list = []
    for t in range(20):
        spikes = neuron(input_current)
        spikes_list.append(spikes)

    spikes_tensor = torch.stack(spikes_list, dim=-1)
    print(f"Input: {input_current.shape} | Output: {spikes_tensor.shape}")
    print(f"Firing rate: {spikes_tensor.mean():.3f}")

    # Test 2: STDP Learning
    print("\n--- Test 2: STDP Learning ---")
    stdp = STDPLearner(n_pre=100, n_post=50)

    for t in range(100):
        pre = (torch.rand(100) > 0.8).float()
        post = (torch.rand(50) > 0.9).float()
        reward = torch.tensor(1.0 if torch.rand(1) > 0.5 else -1.0)
        stdp(pre, post, reward)

    stats = stdp.get_weight_stats()
    print(f"Weight mean: {stats['mean']:.4f} | std: {stats['std']:.4f}")

    # Test 3: Time-to-Spike Encoder
    print("\n--- Test 3: T2S Encoder ---")
    encoder = TimeToSpikeEncoder(timesteps=15)
    values = torch.tensor([0.0, 0.25, 0.5, 0.75, 1.0])
    spikes = encoder(values.unsqueeze(0))
    decoded = encoder.decode(spikes)
    print(f"Input: {values.tolist()}")
    print(f"Decoded: {decoded.squeeze().tolist()}")

    # Test 4: Spiking MLP
    print("\n--- Test 4: Spiking MLP ---")
    mlp = SpikingMLP(input_dim=784, hidden_dims=[256, 128], output_dim=10)
    x = torch.randn(32, 784)
    output, spike_trains = mlp(x, return_spikes=True)
    print(f"Input: {x.shape} | Output: {output.shape}")

    # Test 5: SNN-ViT (small)
    print("\n--- Test 5: SNN-ViT ---")
    vit = SpikingViT(
        image_size=64, patch_size=8, d_model=128,
        n_heads=4, n_layers=4, n_classes=10, timesteps=10,
    )
    x = torch.randn(2, 3, 64, 64)
    output = vit(x)
    print(f"Input: {x.shape} | Output: {output.shape}")

    # Test 6: Spiking GPT-2 (small)
    print("\n--- Test 6: Spiking GPT-2 ---")
    gpt2 = SpikingGPT2(
        d_model=128, n_heads=4, n_layers=4,
        d_ff=256, vocab_size=1000, timesteps=10,
    )
    idx = torch.randint(0, 1000, (2, 32))
    logits, loss = gpt2(idx, targets=idx)
    print(f"Input: {idx.shape} | Logits: {logits.shape} | Loss: {loss.item():.4f}")

    generated = gpt2.generate(idx[:, :5], max_new_tokens=10)
    print(f"Generated: {generated.shape}")

    print("\n" + "=" * 70)
    print("All SNN modules tested successfully!")
    print("=" * 70)
