use rusqlite::Connection;

use super::nt_memory_store::{insert_or_get_node, upsert_edge};
use super::nt_memory_types::{NodeType, RelationType};

pub fn seed_foundational_knowledge(conn: &Connection) -> rusqlite::Result<usize> {
    let mut count = 0;

    let mathematics = insert_or_get_node(
        conn,
        "Mathematics",
        NodeType::Concept,
        Some("The abstract science of number, quantity, and space"),
        None,
        Some("mathematics"),
    )?;
    let algebra = insert_or_get_node(
        conn,
        "Algebra",
        NodeType::Concept,
        Some("Study of mathematical symbols and rules for manipulating them"),
        None,
        Some("mathematics"),
    )?;
    let geometry = insert_or_get_node(
        conn,
        "Geometry",
        NodeType::Concept,
        Some("Study of shapes, sizes, and properties of space"),
        None,
        Some("mathematics"),
    )?;
    let calculus = insert_or_get_node(
        conn,
        "Calculus",
        NodeType::Concept,
        Some("Study of continuous change via derivatives and integrals"),
        None,
        Some("mathematics"),
    )?;
    let topology = insert_or_get_node(
        conn,
        "Topology",
        NodeType::Concept,
        Some("Study of properties preserved under continuous deformations"),
        None,
        Some("mathematics"),
    )?;
    let logic = insert_or_get_node(
        conn,
        "Logic",
        NodeType::Concept,
        Some("Study of valid reasoning and argumentation"),
        None,
        Some("philosophy"),
    )?;
    let set_theory = insert_or_get_node(
        conn,
        "Set Theory",
        NodeType::Theory,
        Some("Study of sets as foundational mathematical objects"),
        None,
        Some("mathematics"),
    )?;
    let category_theory = insert_or_get_node(
        conn,
        "Category Theory",
        NodeType::Theory,
        Some("Abstract study of mathematical structures and relationships between them"),
        None,
        Some("mathematics"),
    )?;
    let group_theory = insert_or_get_node(
        conn,
        "Group Theory",
        NodeType::Theory,
        Some("Study of algebraic groups and their symmetries"),
        None,
        Some("mathematics"),
    )?;
    let number_theory = insert_or_get_node(
        conn,
        "Number Theory",
        NodeType::Theory,
        Some("Study of integers and integer-valued functions"),
        None,
        Some("mathematics"),
    )?;
    count += 10;

    upsert_edge(
        conn,
        &algebra,
        &mathematics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &geometry,
        &mathematics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &calculus,
        &mathematics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &topology,
        &mathematics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &set_theory,
        &mathematics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &category_theory,
        &mathematics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &group_theory,
        &algebra,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &number_theory,
        &mathematics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &logic,
        &mathematics,
        RelationType::Related,
        1.0,
        Some("Foundations of mathematics"),
    )?;
    upsert_edge(
        conn,
        &set_theory,
        &logic,
        RelationType::PrerequisiteOf,
        1.0,
        None,
    )?;

    let physics = insert_or_get_node(
        conn,
        "Physics",
        NodeType::Concept,
        Some("Study of matter, energy, and their interactions"),
        None,
        Some("physics"),
    )?;
    let mechanics = insert_or_get_node(
        conn,
        "Classical Mechanics",
        NodeType::Theory,
        Some("Laws of motion and gravitation governing macroscopic objects"),
        None,
        Some("physics"),
    )?;
    let thermodynamics = insert_or_get_node(
        conn,
        "Thermodynamics",
        NodeType::Theory,
        Some("Study of heat, work, and energy transformations"),
        None,
        Some("physics"),
    )?;
    let electromagnetism = insert_or_get_node(
        conn,
        "Electromagnetism",
        NodeType::Theory,
        Some("Study of electric and magnetic fields and their interactions"),
        None,
        Some("physics"),
    )?;
    let relativity = insert_or_get_node(
        conn,
        "Relativity",
        NodeType::Theory,
        Some("Einstein's theories of space, time, and gravity"),
        None,
        Some("physics"),
    )?;
    let quantum_mechanics = insert_or_get_node(
        conn,
        "Quantum Mechanics",
        NodeType::Theory,
        Some("Study of nature at atomic and subatomic scales"),
        None,
        Some("physics"),
    )?;
    let particle_physics = insert_or_get_node(
        conn,
        "Particle Physics",
        NodeType::Theory,
        Some("Study of fundamental particles and forces"),
        None,
        Some("physics"),
    )?;
    let quantum_field_theory = insert_or_get_node(
        conn,
        "Quantum Field Theory",
        NodeType::Theory,
        Some("Theoretical framework combining quantum mechanics with special relativity"),
        None,
        Some("physics"),
    )?;
    let string_theory = insert_or_get_node(conn, "String Theory", NodeType::Theory, Some("Theoretical framework where point-like particles are replaced by one-dimensional strings"), None, Some("physics"))?;
    let e8_theory = insert_or_get_node(conn, "E8 Lie Group", NodeType::Theory, Some("The largest exceptional simple Lie group, 248-dimensional, related to grand unified theories"), None, Some("physics"))?;
    count += 10;

    upsert_edge(
        conn,
        &mechanics,
        &physics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &thermodynamics,
        &physics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &electromagnetism,
        &physics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &relativity,
        &physics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &quantum_mechanics,
        &physics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &particle_physics,
        &physics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &quantum_field_theory,
        &quantum_mechanics,
        RelationType::ExtensionOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &string_theory,
        &quantum_field_theory,
        RelationType::ExtensionOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &e8_theory,
        &string_theory,
        RelationType::Related,
        0.8,
        Some("E8 appears in heterotic string theory"),
    )?;
    upsert_edge(
        conn,
        &calculus,
        &mechanics,
        RelationType::PrerequisiteOf,
        1.0,
        None,
    )?;

    let computer_science = insert_or_get_node(
        conn,
        "Computer Science",
        NodeType::Concept,
        Some("Study of computation, information, and automated reasoning"),
        None,
        Some("computer_science"),
    )?;
    let algorithms = insert_or_get_node(
        conn,
        "Algorithms",
        NodeType::Concept,
        Some("Step-by-step procedures for solving computational problems"),
        None,
        Some("computer_science"),
    )?;
    let data_structures = insert_or_get_node(
        conn,
        "Data Structures",
        NodeType::Concept,
        Some("Ways of organizing and storing data for efficient access"),
        None,
        Some("computer_science"),
    )?;
    let computation_theory = insert_or_get_node(
        conn,
        "Theory of Computation",
        NodeType::Theory,
        Some("Study of what problems can be solved by computers and at what cost"),
        None,
        Some("computer_science"),
    )?;
    let machine_learning = insert_or_get_node(
        conn,
        "Machine Learning",
        NodeType::Concept,
        Some("Algorithms that improve through experience and data"),
        None,
        Some("computer_science"),
    )?;
    let deep_learning = insert_or_get_node(
        conn,
        "Deep Learning",
        NodeType::Method,
        Some("Machine learning using multi-layer neural networks"),
        None,
        Some("computer_science"),
    )?;
    let reinforcement_learning = insert_or_get_node(
        conn,
        "Reinforcement Learning",
        NodeType::Method,
        Some("Learning optimal behavior through trial and error with rewards"),
        None,
        Some("computer_science"),
    )?;
    let natural_language_processing = insert_or_get_node(
        conn,
        "Natural Language Processing",
        NodeType::Concept,
        Some("Computational techniques for understanding and generating human language"),
        None,
        Some("computer_science"),
    )?;
    let computer_vision = insert_or_get_node(
        conn,
        "Computer Vision",
        NodeType::Concept,
        Some("Enabling computers to interpret and understand visual information"),
        None,
        Some("computer_science"),
    )?;
    let information_theory = insert_or_get_node(
        conn,
        "Information Theory",
        NodeType::Theory,
        Some("Study of quantification, storage, and communication of information"),
        None,
        Some("computer_science"),
    )?;
    count += 10;

    upsert_edge(
        conn,
        &algorithms,
        &computer_science,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &data_structures,
        &computer_science,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &computation_theory,
        &computer_science,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &machine_learning,
        &computer_science,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &deep_learning,
        &machine_learning,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &reinforcement_learning,
        &machine_learning,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &natural_language_processing,
        &computer_science,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &computer_vision,
        &computer_science,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &information_theory,
        &computer_science,
        RelationType::Related,
        0.8,
        None,
    )?;
    upsert_edge(
        conn,
        &computation_theory,
        &mathematics,
        RelationType::PrerequisiteOf,
        0.7,
        None,
    )?;

    let philosophy = insert_or_get_node(
        conn,
        "Philosophy",
        NodeType::Concept,
        Some("Study of fundamental questions about existence, knowledge, and values"),
        None,
        Some("philosophy"),
    )?;
    let epistemology = insert_or_get_node(
        conn,
        "Epistemology",
        NodeType::Theory,
        Some("Study of knowledge, belief, and justification"),
        None,
        Some("philosophy"),
    )?;
    let ontology = insert_or_get_node(
        conn,
        "Ontology",
        NodeType::Theory,
        Some("Study of being, existence, and reality"),
        None,
        Some("philosophy"),
    )?;
    let ethics = insert_or_get_node(
        conn,
        "Ethics",
        NodeType::Theory,
        Some("Study of moral principles and values"),
        None,
        Some("philosophy"),
    )?;
    let consciousness = insert_or_get_node(
        conn,
        "Consciousness",
        NodeType::Concept,
        Some("Subjective experience and awareness of self and environment"),
        None,
        Some("philosophy"),
    )?;
    let phenomenology = insert_or_get_node(
        conn,
        "Phenomenology",
        NodeType::Method,
        Some("Philosophical study of structures of subjective experience"),
        None,
        Some("philosophy"),
    )?;
    count += 6;

    upsert_edge(
        conn,
        &epistemology,
        &philosophy,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &ontology,
        &philosophy,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &ethics,
        &philosophy,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &consciousness,
        &philosophy,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &phenomenology,
        &philosophy,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &epistemology,
        &machine_learning,
        RelationType::Related,
        0.7,
        Some("ML as inductive inference"),
    )?;

    let neuroscience = insert_or_get_node(
        conn,
        "Neuroscience",
        NodeType::Concept,
        Some("Scientific study of the nervous system and brain function"),
        None,
        Some("neuroscience"),
    )?;
    let cognitive_science = insert_or_get_node(
        conn,
        "Cognitive Science",
        NodeType::Concept,
        Some("Interdisciplinary study of mind and intelligence"),
        None,
        Some("cognitive_science"),
    )?;
    let global_workspace_theory = insert_or_get_node(
        conn,
        "Global Workspace Theory",
        NodeType::Theory,
        Some("Theory of consciousness where specialized modules compete for global broadcast"),
        None,
        Some("cognitive_science"),
    )?;
    let integrated_information_theory = insert_or_get_node(
        conn,
        "Integrated Information Theory",
        NodeType::Theory,
        Some("Theory of consciousness based on integrated information (phi)"),
        None,
        Some("neuroscience"),
    )?;
    let predictive_coding = insert_or_get_node(conn, "Predictive Coding", NodeType::Theory, Some("Theory that the brain constantly generates and updates predictions of nt_world_sense input"), None, Some("neuroscience"))?;
    let free_energy_principle = insert_or_get_node(
        conn,
        "Free Energy Principle",
        NodeType::Theory,
        Some("Theory that organisms minimize variational free energy to maintain their states"),
        None,
        Some("neuroscience"),
    )?;
    count += 6;

    upsert_edge(
        conn,
        &cognitive_science,
        &neuroscience,
        RelationType::Related,
        0.8,
        None,
    )?;
    upsert_edge(
        conn,
        &global_workspace_theory,
        &cognitive_science,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &integrated_information_theory,
        &neuroscience,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &predictive_coding,
        &neuroscience,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &free_energy_principle,
        &neuroscience,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;

    let neuroscience: String = conn
        .query_row(
            "SELECT id FROM nodes WHERE title='Neuroscience' LIMIT 1",
            [],
            |r| -> rusqlite::Result<String> { r.get(0) },
        )
        .unwrap_or_default();
    if !neuroscience.is_empty() {
        upsert_edge(
            conn,
            &consciousness,
            &neuroscience,
            RelationType::Related,
            0.9,
            Some("Neural correlates of consciousness"),
        )?;
    }

    let vsa = insert_or_get_node(conn, "Vector Symbolic Architecture", NodeType::Theory, Some("Computational framework using high-dimensional distributed representations and algebraic operations"), None, Some("computer_science"))?;
    let hyperdimensional_computing = insert_or_get_node(
        conn,
        "Hyperdimensional Computing",
        NodeType::Method,
        Some("Computing paradigm using very high-dimensional vectors as representations"),
        None,
        Some("computer_science"),
    )?;
    let holographic_reduced = insert_or_get_node(
        conn,
        "Holographic Reduced Representations",
        NodeType::Algorithm,
        Some("VSA model using circular convolution for binding"),
        None,
        Some("computer_science"),
    )?;
    let map_vsa = insert_or_get_node(
        conn,
        "Multiply-Add-Permute (MAP)",
        NodeType::Algorithm,
        Some("VSA model using element-wise multiplication for binding"),
        None,
        Some("computer_science"),
    )?;
    let hypercube_core = insert_or_get_node(
        conn,
        "HyperCube (NeoTrix)",
        NodeType::Method,
        Some("4096-dimensional VSA engine with 16 semantic axes, bind/bundle/permute operations"),
        None,
        Some("neotrix"),
    )?;
    count += 5;

    upsert_edge(
        conn,
        &vsa,
        &hyperdimensional_computing,
        RelationType::Related,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &holographic_reduced,
        &vsa,
        RelationType::InstanceOf,
        1.0,
        None,
    )?;
    upsert_edge(conn, &map_vsa, &vsa, RelationType::InstanceOf, 1.0, None)?;
    upsert_edge(
        conn,
        &hypercube_core,
        &vsa,
        RelationType::InstanceOf,
        1.0,
        Some("NeoTrix implements MAP-based VSA"),
    )?;
    upsert_edge(
        conn,
        &hypercube_core,
        &global_workspace_theory,
        RelationType::Related,
        0.8,
        Some("HyperCube feeds GWT attention router"),
    )?;

    let e8_neotrix = insert_or_get_node(conn, "E8 Reasoning Engine", NodeType::Method, Some("64-state deterministic state machine over 6 binary reasoning axes, isomorphic to I Ching"), None, Some("neotrix"))?;
    let seal = insert_or_get_node(conn, "SEAL Self-Iterating Loop", NodeType::Method, Some("16-stage closed-loop pipeline for self-improvement: snapshot→gap→edit→apply→reward→absorb"), None, Some("neotrix"))?;
    let gwt_neotrix = insert_or_get_node(conn, "GWT Attention Router", NodeType::Method, Some("11 specialist modules competing for global workspace broadcast via salience computation"), None, Some("neotrix"))?;
    let metacognition = insert_or_get_node(
        conn,
        "MetaCognition Layer",
        NodeType::Method,
        Some("Self-awareness system: CodeScanner, WeaknessAnalyzer, MetaMonitor, EvolutionPlanner"),
        None,
        Some("neotrix"),
    )?;
    count += 4;

    upsert_edge(
        conn,
        &e8_neotrix,
        &e8_theory,
        RelationType::InspiredBy,
        0.7,
        Some("E8 Lie algebra inspires the 64-state reasoning topology"),
    )?;
    upsert_edge(
        conn,
        &e8_neotrix,
        &hypercube_core,
        RelationType::Related,
        0.8,
        Some("E8 reasoning state feeds into HyperCube queries"),
    )?;
    upsert_edge(
        conn,
        &seal,
        &e8_neotrix,
        RelationType::ExtensionOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &gwt_neotrix,
        &global_workspace_theory,
        RelationType::InstanceOf,
        1.0,
        Some("NeoTrix implements computational GWT"),
    )?;
    upsert_edge(
        conn,
        &metacognition,
        &consciousness,
        RelationType::Related,
        0.8,
        Some("Self-awareness via metacognitive monitoring"),
    )?;
    upsert_edge(
        conn,
        &metacognition,
        &seal,
        RelationType::Related,
        0.9,
        Some("MetaCognition drives SEAL evolution planning"),
    )?;

    let attention = insert_or_get_node(
        conn,
        "Attention Mechanism",
        NodeType::Method,
        Some("Computational mechanism that selectively focuses on relevant information"),
        None,
        Some("machine_learning"),
    )?;
    let transformer = insert_or_get_node(
        conn,
        "Transformer Architecture",
        NodeType::Framework,
        Some("Neural network architecture based on self-attention, foundational to modern LLMs"),
        None,
        Some("deep_learning"),
    )?;
    let llm = insert_or_get_node(
        conn,
        "Large Language Model",
        NodeType::Framework,
        Some("Transformer-based neural network trained on massive text corpora"),
        None,
        Some("deep_learning"),
    )?;
    let agent_framework = insert_or_get_node(
        conn,
        "AI Agent",
        NodeType::Concept,
        Some("Autonomous system that perceives, reasons, plans, and acts to achieve goals"),
        None,
        Some("ai"),
    )?;
    let rag = insert_or_get_node(
        conn,
        "Retrieval-Augmented Generation",
        NodeType::Method,
        Some("Augmenting LLMs with external knowledge retrieval"),
        None,
        Some("nlp"),
    )?;
    let mcp = insert_or_get_node(
        conn,
        "Model Context Protocol",
        NodeType::Framework,
        Some("Open protocol for connecting LLMs to external tools and data sources"),
        None,
        Some("ai"),
    )?;
    count += 6;

    upsert_edge(
        conn,
        &transformer,
        &attention,
        RelationType::DependsOn,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &llm,
        &transformer,
        RelationType::InstanceOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &agent_framework,
        &llm,
        RelationType::DependsOn,
        0.9,
        Some("Modern agents use LLMs as cognitive cores"),
    )?;
    upsert_edge(conn, &rag, &llm, RelationType::ExtensionOf, 1.0, None)?;
    upsert_edge(
        conn,
        &mcp,
        &agent_framework,
        RelationType::Supports,
        1.0,
        Some("MCP enables standardized agent-tool communication"),
    )?;
    upsert_edge(
        conn,
        &agent_framework,
        &e8_neotrix,
        RelationType::Related,
        0.7,
        Some("NeoTrix agents use E8 reasoning"),
    )?;
    upsert_edge(
        conn,
        &agent_framework,
        &reinforcement_learning,
        RelationType::Related,
        0.7,
        Some("RL for agent optimization"),
    )?;

    let neurosymbolic = insert_or_get_node(
        conn,
        "Neuro-Symbolic AI",
        NodeType::Method,
        Some("Integration of neural networks with symbolic reasoning"),
        None,
        Some("ai"),
    )?;
    let _graph_neural = insert_or_get_node(
        conn,
        "Graph Neural Network",
        NodeType::Algorithm,
        Some("Neural network operating on graph-structured data"),
        None,
        Some("deep_learning"),
    )?;
    let diffusion = insert_or_get_node(
        conn,
        "Diffusion Model",
        NodeType::Algorithm,
        Some("Generative model that learns to denoise data"),
        None,
        Some("deep_learning"),
    )?;
    let nt_world_model = insert_or_get_node(
        conn,
        "World Model",
        NodeType::Method,
        Some("Internal model of how the environment works, used for planning and prediction"),
        None,
        Some("reinforcement_learning"),
    )?;
    let jepa = insert_or_get_node(
        conn,
        "Joint Embedding Predictive Architecture",
        NodeType::Algorithm,
        Some("Self-supervised learning by predicting representations in embedding space"),
        None,
        Some("deep_learning"),
    )?;
    let mamba = insert_or_get_node(
        conn,
        "Mamba State Space Model",
        NodeType::Algorithm,
        Some("Selective state-space model for efficient long-sequence modeling"),
        None,
        Some("deep_learning"),
    )?;
    let lora_t = insert_or_get_node(
        conn,
        "LoRA Low-Rank Adaptation",
        NodeType::Method,
        Some("Parameter-efficient fine-tuning using low-rank matrix decomposition"),
        None,
        Some("deep_learning"),
    )?;
    count += 7;

    upsert_edge(
        conn,
        &neurosymbolic,
        &machine_learning,
        RelationType::Related,
        0.8,
        None,
    )?;
    upsert_edge(
        conn,
        &neurosymbolic,
        &logic,
        RelationType::DependsOn,
        0.7,
        None,
    )?;
    upsert_edge(
        conn,
        &neurosymbolic,
        &vsa,
        RelationType::Related,
        0.8,
        Some("VSA as neuro-symbolic bridge"),
    )?;
    upsert_edge(
        conn,
        &diffusion,
        &deep_learning,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &nt_world_model,
        &reinforcement_learning,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &jepa,
        &nt_world_model,
        RelationType::ExtensionOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &mamba,
        &deep_learning,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &lora_t,
        &deep_learning,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;

    let biology = insert_or_get_node(
        conn,
        "Biology",
        NodeType::Concept,
        Some("Study of living organisms and life processes"),
        None,
        Some("biology"),
    )?;
    let evolution_t = insert_or_get_node(
        conn,
        "Evolution",
        NodeType::Theory,
        Some("Process of change in all forms of life over generations via natural selection"),
        None,
        Some("biology"),
    )?;
    let genetics = insert_or_get_node(
        conn,
        "Genetics",
        NodeType::Theory,
        Some("Study of genes, heredity, and genetic variation"),
        None,
        Some("biology"),
    )?;
    let neuroscience_bio = insert_or_get_node(
        conn,
        "Neuroscience",
        NodeType::Concept,
        Some("Study of the nervous system including brain, spinal cord, and neural circuits"),
        None,
        Some("biology"),
    )?;
    let network_science = insert_or_get_node(
        conn,
        "Network Science",
        NodeType::Theory,
        Some("Study of complex networks including social, biological, and technological networks"),
        None,
        Some("computer_science"),
    )?;
    let complexity = insert_or_get_node(conn, "Complexity Theory", NodeType::Theory, Some("Study of complex systems with many interacting components exhibiting emergent behavior"), None, Some("computer_science"))?;
    let emergence = insert_or_get_node(
        conn,
        "Emergence",
        NodeType::Concept,
        Some("Phenomenon where complex patterns arise from simple interactions"),
        None,
        Some("philosophy"),
    )?;
    let entropy = insert_or_get_node(
        conn,
        "Entropy",
        NodeType::Concept,
        Some("Measure of disorder or uncertainty in a system"),
        None,
        Some("thermodynamics"),
    )?;
    count += 8;

    upsert_edge(
        conn,
        &evolution_t,
        &biology,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &genetics,
        &biology,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &neuroscience_bio,
        &biology,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &network_science,
        &mathematics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &complexity,
        &network_science,
        RelationType::Related,
        0.9,
        None,
    )?;
    upsert_edge(
        conn,
        &complexity,
        &emergence,
        RelationType::Related,
        0.9,
        None,
    )?;
    upsert_edge(
        conn,
        &entropy,
        &thermodynamics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &entropy,
        &information_theory,
        RelationType::Related,
        0.9,
        Some("Shannon entropy"),
    )?;
    upsert_edge(
        conn,
        &emergence,
        &consciousness,
        RelationType::Related,
        0.7,
        Some("Consciousness as emergent property"),
    )?;
    upsert_edge(
        conn,
        &evolution_t,
        &machine_learning,
        RelationType::Related,
        0.7,
        Some("Evolutionary algorithms"),
    )?;

    let karpathy_autoresearch = insert_or_get_node(conn, "Karpathy AutoResearch", NodeType::Repository, Some("Recursive research loop: agent modifies code, runs test, evaluates, commits if improved"), Some("https://github.com/karpathy/AutoResearch"), Some("github.com"))?;
    let hermes_agent = insert_or_get_node(conn, "Hermes Agent", NodeType::Repository, Some("Open-source self-improving AI agent framework with closed-loop learning and skill generation"), Some("https://github.com/NousResearch/hermes-agent"), Some("github.com"))?;
    let geneclaw = insert_or_get_node(
        conn,
        "Geneclaw",
        NodeType::Repository,
        Some("Safe auditable self-evolving agent framework with 5-stage gate protocol"),
        Some("https://github.com/geneclaw/geneclaw"),
        Some("github.com"),
    )?;
    let dgm = insert_or_get_node(conn, "Darwin Gödel Machine", NodeType::Paper, Some("Framework for open-ended self-improvement of coding agents through self-modification and archive growth"), Some("https://arxiv.org/abs/2603.19461"), Some("arxiv.org"))?;
    let _molmon = insert_or_get_node(
        conn,
        "MOLTRON",
        NodeType::Repository,
        Some(
            "Self-evolving agent framework enabling agents to build and evolve skills autonomously",
        ),
        Some("https://github.com/adridder/moltron"),
        Some("github.com"),
    )?;
    let _claude_mem = insert_or_get_node(
        conn,
        "claude-mem",
        NodeType::Repository,
        Some("Cross-session memory plugin for Claude Code using SQLite and ChromaDB"),
        Some("https://github.com/thedotmack/claude-mem"),
        Some("github.com"),
    )?;
    let mem0 = insert_or_get_node(
        conn,
        "Mem0",
        NodeType::Repository,
        Some("Memory infrastructure for AI agents with LoCoMo/LongMemEval benchmarks"),
        Some("https://github.com/mem0ai/mem0"),
        Some("github.com"),
    )?;
    count += 7;

    upsert_edge(
        conn,
        &karpathy_autoresearch,
        &dgm,
        RelationType::InspiredBy,
        0.9,
        None,
    )?;
    upsert_edge(
        conn,
        &dgm,
        &seal,
        RelationType::Related,
        0.7,
        Some("Both implement recursive self-improvement loops"),
    )?;
    upsert_edge(
        conn,
        &hermes_agent,
        &seal,
        RelationType::Related,
        0.7,
        Some("Comparable self-evolution architecture"),
    )?;
    upsert_edge(
        conn,
        &geneclaw,
        &seal,
        RelationType::Related,
        0.6,
        Some("Geneclaw gate protocol vs SEAL 16-stage pipeline"),
    )?;
    upsert_edge(
        conn,
        &mem0,
        &hypercube_core,
        RelationType::Related,
        0.6,
        Some("Memory systems for AI agents"),
    )?;

    let _linguistics = insert_or_get_node(
        conn,
        "Linguistics",
        NodeType::Concept,
        Some("Scientific study of language structure and use"),
        None,
        Some("linguistics"),
    )?;
    let _psychology = insert_or_get_node(
        conn,
        "Psychology",
        NodeType::Concept,
        Some("Study of mind and behavior"),
        None,
        Some("psychology"),
    )?;
    let _sociology = insert_or_get_node(
        conn,
        "Sociology",
        NodeType::Concept,
        Some("Study of society and social behavior"),
        None,
        Some("sociology"),
    )?;
    let economics = insert_or_get_node(
        conn,
        "Economics",
        NodeType::Concept,
        Some("Study of production, consumption, and transfer of wealth"),
        None,
        Some("economics"),
    )?;
    let game_theory = insert_or_get_node(
        conn,
        "Game Theory",
        NodeType::Theory,
        Some("Study of strategic decision-making between rational agents"),
        None,
        Some("economics"),
    )?;
    let cybernetics = insert_or_get_node(
        conn,
        "Cybernetics",
        NodeType::Theory,
        Some("Study of regulatory systems, feedback, and control in complex systems"),
        None,
        Some("systems_science"),
    )?;
    let systems_theory = insert_or_get_node(
        conn,
        "Systems Theory",
        NodeType::Theory,
        Some("Interdisciplinary study of systems and their emergent properties"),
        None,
        Some("systems_science"),
    )?;
    let dynamical_systems = insert_or_get_node(
        conn,
        "Dynamical Systems Theory",
        NodeType::Theory,
        Some("Study of systems that evolve over time according to deterministic rules"),
        None,
        Some("mathematics"),
    )?;
    let chaos_theory = insert_or_get_node(
        conn,
        "Chaos Theory",
        NodeType::Theory,
        Some("Study of dynamical systems highly sensitive to initial conditions"),
        None,
        Some("mathematics"),
    )?;
    count += 9;

    upsert_edge(
        conn,
        &game_theory,
        &economics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &game_theory,
        &reinforcement_learning,
        RelationType::Related,
        0.9,
        Some("RL as single-agent game theory"),
    )?;
    upsert_edge(
        conn,
        &cybernetics,
        &systems_theory,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &cybernetics,
        &free_energy_principle,
        RelationType::Related,
        0.7,
        Some("FEP as Bayesian cybernetics"),
    )?;
    upsert_edge(
        conn,
        &dynamical_systems,
        &mathematics,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;
    upsert_edge(
        conn,
        &chaos_theory,
        &dynamical_systems,
        RelationType::SubclassOf,
        1.0,
        None,
    )?;

    Ok(count)
}
