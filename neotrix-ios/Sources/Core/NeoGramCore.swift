// NeoGramCore - Main app coordinator
// Mirrors Telegram's ApplicationContext + AccountContext pattern

import Foundation
import Combine
import NeoTrixFFI

public final class NeoGramCore: ObservableObject {
    public static let shared = NeoGramCore()
    
    private var handle: NeoTrixHandle?
    private var config: NeoTrixConfig
    
    private init() {
        self.config = NeoTrixConfig(
            serverUrl: "https://api.neotrix.ai",
            apiKey: "",
            enableAiFeatures: true,
            enablePremiumFeatures: true,
            logLevel: "info",
            dataDirectory: NSSearchPathForDirectoriesInDomains(.documentDirectory, .userDomainMask, true).first!,
            cacheSizeMb: 512
        )
    }
    
    public func initialize() async throws {
        let result = try await NeoTrixFFI.neotrixInitialize(config: config)
        self.handle = result
        let caps = NeoTrixFFI.neotrixCapabilities(handle: result)
        print("NeoGramCore initialized with capabilities: \(caps.e8Reasoning)")
    }
    
    public var e8Reasoning: E8ReasoningImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixE8Reasoning(handle: handle)
    }
    
    public var vsaHyperCube: VsaHyperCubeImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixVsaHypercube(handle: handle)
    }
    
    public var gwtAttention: GwtAttentionRouterImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixGwtAttention(handle: handle)
    }
    
    public var consciousnessTree: ConsciousnessTreeImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixConsciousnessTree(handle: handle)
    }
    
    public var sealPipeline: SealPipelineImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixSealPipeline(handle: handle)
    }
    
    public var kbBridge: KbBridgeImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixKbBridge(handle: handle)
    }
    
    public var skillTree: SkillTreeImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixSkillTree(handle: handle)
    }
    
    public var runeSocketing: RuneSocketingImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixRuneSocketing(handle: handle)
    }
    
    public var constellationSystem: ConstellationSystemImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixConstellationSystem(handle: handle)
    }
    
    public var dualSpecialization: DualSpecializationImpl? {
        guard let handle = handle else { return nil }
        return try? NeoTrixFFI.neotrixDualSpecialization(handle: handle)
    }
    
    public func capabilities() -> CapabilityList? {
        guard let handle = handle else { return nil }
        return NeoTrixFFI.neotrixCapabilities(handle: handle)
    }
    
    public func healthCheck() -> HealthStatus? {
        guard let handle = handle else { return nil }
        return NeoTrixFFI.neotrixHealthCheck(handle: handle)
    }
    
    public func shutdown() {
        if let handle = handle {
            _ = NeoTrixFFI.neotrixShutdown(handle: handle)
            self.handle = nil
        }
    }
}