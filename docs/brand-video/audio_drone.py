#!/usr/bin/env python3
"""Generate ambient drone audio for NeoTrix manifesto video.
Output: 44.1kHz, 16-bit, mono WAV, ~62 seconds.
Zero external dependencies (numpy + wave only).
"""
import numpy as np
import wave
import struct
import math

SR = 44100
DURATION = 62  # seconds (slightly longer than video)
AMP = 0.30     # master volume

t = np.linspace(0, DURATION, int(SR * DURATION), endpoint=False)

# --- Layer 1: Deep bass drone (A1 = 55 Hz) ---
bass = np.sin(2 * np.pi * 55.0 * t)
# Slow volume pulse
bass_env = 1.0 - 0.3 * np.sin(2 * np.pi * 0.05 * t)
bass = bass * bass_env * 0.35

# --- Layer 2: Low pad (A2 = 110 Hz + detune) ---
pad110 = np.sin(2 * np.pi * 110.0 * t)
# Detuned companion
pad113 = np.sin(2 * np.pi * 113.0 * t)
# Slow LFO for shimmer
pad_lfo = 1.0 + 0.15 * np.sin(2 * np.pi * 0.08 * t)
pad = (pad110 + pad113) * 0.5 * pad_lfo * 0.25

# --- Layer 3: Mid shimmer (E4 = 329.6 Hz + gentle vibrato) ---
vibrato = 2 * np.pi * (329.6 + 3.0 * np.sin(2 * np.pi * 0.15 * t)) * t
shimmer = np.sin(vibrato)
# Fade in/out
shimmer_env = np.minimum(1.0, t / 5.0) * np.minimum(1.0, (DURATION - t) / 3.0)
shimmer = shimmer * shimmer_env * 0.15

# --- Layer 4: High ethereal (A5 = 880 Hz + slow tremolo) ---
ether = np.sin(2 * np.pi * 880.0 * t)
tremolo = 0.5 + 0.5 * np.sin(2 * np.pi * 0.03 * t)
ether = ether * tremolo * 0.10

# --- Layer 5: Sub-bass rumble (27.5 Hz, very quiet) ---
sub = np.sin(2 * np.pi * 27.5 * t) * 0.08

# --- Master mix ---
mix = bass + pad + shimmer + ether + sub

# Limit to prevent clipping
mix = np.tanh(mix * 1.5) * AMP

# Convert to 16-bit PCM
mix_int = np.clip(mix * 32767, -32768, 32767).astype(np.int16)

out_path = "bg_scenes/ambient_drone.wav"
with wave.open(out_path, 'w') as wf:
    wf.setnchannels(1)
    wf.setsampwidth(2)
    wf.setframerate(SR)
    wf.writeframes(mix_int.tobytes())

print(f"Generated: {out_path} ({len(mix_int)/SR:.1f}s, {SR}Hz, 16-bit mono)")
