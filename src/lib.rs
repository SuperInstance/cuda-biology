/*!
# cuda-biology

Biological agent runtime — maps instinct engine to instruction set.

The complete pipeline:
```text
```no_run
Environment → Sensors → Membrane → Enzymes → Genes → RNA → Proteins → FLUX bytecode → Action → Feedback
```no_run

Every operation costs ATP. Rest instinct generates ATP. Circadian rhythm
modulates instinct strength. Apoptosis terminates agents that can't sustain themselves.
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The 10 biological instincts with energy profiles
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Instinct {
    Survive,    // HALT, TRAP, RESOURCE_ACQUIRE — costs 0, generates nothing
    Perceive,   // IO_READ, SENSOR_ACQUIRE, FUSE_CONF — cheap sensing
    Navigate,   // JMP, CALL, RET — movement through state space
    Communicate,// TELL, ASK, BROADCAST — agent messaging
    Learn,      // BOX, UNBOX, REGION_CREATE — memory formation
    Defend,     // MEMBRANE_CHK, VERIFY, CAP_REQ — security
    Rest,       // ATP_GEN — generates energy, no action
    Play,       // Explore unknown states, try new gene combinations
    Create,     // Compose new genes from existing patterns
    Socialize,  // TRUST_UPDATE, DELEGATE — fleet coordination
}

impl Instinct {
    pub fn all() -> &'static [Instinct] {
        &[Instinct::Survive, Instinct::Perceive, Instinct::Navigate, Instinct::Communicate,
          Instinct::Learn, Instinct::Defend, Instinct::Rest, Instinct::Play,
          Instinct::Create, Instinct::Socialize]
    }

    pub fn id(self) -> u8 {
        match self {
            Instinct::Survive => 0, Instinct::Perceive => 1, Instinct::Navigate => 2,
            Instinct::Communicate => 3, Instinct::Learn => 4, Instinct::Defend => 5,
            Instinct::Rest => 6, Instinct::Play => 7, Instinct::Create => 8,
            Instinct::Socialize => 9,
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Instinct::Survive), 1 => Some(Instinct::Perceive),
            2 => Some(Instinct::Navigate), 3 => Some(Instinct::Communicate),
            4 => Some(Instinct::Learn), 5 => Some(Instinct::Defend),
            6 => Some(Instinct::Rest), 7 => Some(Instinct::Play),
            8 => Some(Instinct::Create), 9 => Some(Instinct::Socialize),
            _ => None,
        }
    }

    /// Base energy cost per activation cycle
    pub fn energy_cost(self) -> f64 {
        match self {
            Instinct::Survive => 0.0,
            Instinct::Perceive => 0.3,
            Instinct::Navigate => 0.5,
            Instinct::Communicate => 0.8,
            Instinct::Learn => 0.6,
            Instinct::Defend => 0.2,
            Instinct::Rest => -1.0,   // generates ATP
            Instinct::Play => 0.7,
            Instinct::Create => 1.2,
            Instinct::Socialize => 0.4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Instinct::Survive => "survive", Instinct::Perceive => "perceive",
            Instinct::Navigate => "navigate", Instinct::Communicate => "communicate",
            Instinct::Learn => "learn", Instinct::Defend => "defend",
            Instinct::Rest => "rest", Instinct::Play => "play",
            Instinct::Create => "create", Instinct::Socialize => "socialize",
        }
    }
}

/// A gene — the fundamental unit of behavioral patterns
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gene {
    pub name: String,
    /// Which instinct(s) this gene serves
    pub instinct: Instinct,
    /// How well this gene matches its signal pattern [0,1]
    pub signal_affinity: f64,
    /// How strongly this gene is expressed [0,1]
    pub expression: f64,
    /// Accumulated fitness score from outcomes
    pub fitness: f64,
    /// How many times this gene has been activated
    pub use_count: u32,
    /// How many times activation led to success
    pub success_count: u32,
    /// The behavioral pattern encoded as instruction bytes
    pub bytecode: Vec<u8>,
    /// Confidence in this gene's effectiveness
    pub confidence: f64,
}

impl Gene {
    pub fn new(name: &str, instinct: Instinct) -> Self {
        Gene {
            name: name.to_string(),
            instinct,
            signal_affinity: 0.5,
            expression: 0.5,
            fitness: 0.5,
            use_count: 0,
            success_count: 0,
            bytecode: vec![],
            confidence: 0.5,
        }
    }

    /// Check if gene should be auto-quarantined
    /// Conditions: fitness < 0.1 AND used > 10 times AND success rate < 15%
    pub fn should_quarantine(&self) -> bool {
        if self.use_count < 10 { return false; }
        let success_rate = if self.use_count > 0 { self.success_count as f64 / self.use_count as f64 } else { 0.0 };
        self.fitness < 0.1 && success_rate < 0.15
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.use_count == 0 { return 0.0; }
        self.success_count as f64 / self.use_count as f64
    }

    /// Update fitness based on outcome
    pub fn record_outcome(&mut self, success: bool) {
        self.use_count += 1;
        if success { self.success_count += 1; }
        // Exponential moving average
        let rate = self.success_rate();
        let alpha = 0.1;
        self.fitness = self.fitness * (1.0 - alpha) + rate * alpha;
        // Update confidence based on consistency
        if self.use_count > 5 {
            let variance = (rate - 0.5).abs() * 2.0; // 0=consistent 50%, 1=always or never
            self.confidence = self.confidence * 0.9 + (1.0 - variance) * 0.1;
        }
    }
}

/// An enzyme — matches signals to genes for activation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enzyme {
    pub name: String,
    /// Signal pattern this enzyme responds to
    pub signal_pattern: Vec<u8>,
    /// Genes this enzyme can activate
    pub target_genes: Vec<String>,
    /// Binding threshold — signal must exceed this to bind
    pub threshold: f64,
}

impl Enzyme {
    pub fn new(name: &str, pattern: Vec<u8>, genes: Vec<&str>) -> Self {
        Enzyme { name: name.to_string(), signal_pattern: pattern, target_genes: genes.iter().map(|s| s.to_string()).collect(), threshold: 0.3 }
    }

    /// Try to bind a signal. Returns binding strength [0,1].
    pub fn try_bind(&self, signal: &[u8]) -> f64 {
        if signal.len() != self.signal_pattern.len() { return 0.0; }
        let matches = signal.iter().zip(self.signal_pattern.iter()).filter(|(s,p)| s == p).count();
        let strength = matches as f64 / signal.len() as f64;
        if strength >= self.threshold { strength } else { 0.0 }
    }
}

/// RNA messenger — translates gene into executable protein (bytecode)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RnaMessenger {
    pub source_gene: String,
    pub translated_bytecode: Vec<u8>,
    pub expression_level: f64,
    pub confidence: f64,
}

impl RnaMessenger {
    pub fn translate(gene: &Gene) -> Self {
        RnaMessenger {
            source_gene: gene.name.clone(),
            translated_bytecode: gene.bytecode.clone(),
            expression_level: gene.expression,
            confidence: gene.confidence,
        }
    }
}

/// Membrane — self/other boundary with antibody security
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Membrane {
    pub antibodies: Vec<MembraneAntibody>,
}

/// An antibody that blocks dangerous signals
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MembraneAntibody {
    pub pattern: Vec<u8>,
    pub reason: String,
}

impl Membrane {
    pub fn new() -> Self { Membrane { antibodies: vec![] } }

    pub fn add_antibody(&mut self, pattern: Vec<u8>, reason: &str) {
        self.antibodies.push(MembraneAntibody { pattern, reason: reason.to_string() });
    }

    /// Default dangerous patterns
    pub fn default_antibodies() -> Self {
        let mut m = Membrane::new();
        // Block obvious dangerous operations encoded as byte patterns
        m.add_antibody(b"rm -rf".to_vec(), "destructive filesystem operation");
        m.add_antibody(b"format".to_vec(), "disk format");
        m.add_antibody(b"drop_all".to_vec(), "database destruction");
        m.add_antibody(b"DELETE FROM".to_vec(), "SQL injection");
        m.add_antibody(b"sudo rm".to_vec(), "privileged deletion");
        m
    }

    /// Check if a signal passes the membrane. Returns true if safe.
    pub fn check(&self, signal: &[u8]) -> bool {
        for ab in &self.antibodies {
            if signal.len() >= ab.pattern.len() {
                for i in 0..=signal.len() - ab.pattern.len() {
                    if &signal[i..i+ab.pattern.len()] == ab.pattern.as_slice() {
                        return false; // blocked
                    }
                }
            }
        }
        true
    }
}

/// The biological agent — complete pipeline from instinct to action
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiologicalAgent {
    pub id: String,
    /// Current energy (ATP)
    pub energy: f64,
    pub max_energy: f64,
    /// Gene pool
    pub genes: HashMap<String, Gene>,
    /// Enzymes
    pub enzymes: Vec<Enzyme>,
    /// Membrane
    pub membrane: Membrane,
    /// Circadian phase (0.0-24.0 hours)
    pub circadian_hour: f64,
    /// Whether apoptosis has triggered
    pub dead: bool,
    /// Death reason
    pub death_reason: String,
    /// Activity log
    pub log: Vec<String>,
    /// Consecutive low-energy ticks
    pub low_energy_ticks: u32,
    /// Apoptosis threshold
    pub apoptosis_patience: u32,
    /// Total actions taken
    pub actions_taken: u64,
    pub successful_actions: u64,
}

impl BiologicalAgent {
    pub fn new(id: &str, max_energy: f64) -> Self {
        BiologicalAgent {
            id: id.to_string(),
            energy: max_energy,
            max_energy,
            genes: HashMap::new(),
            enzymes: vec![],
            membrane: Membrane::default_antibodies(),
            circadian_hour: 12.0,
            dead: false,
            death_reason: String::new(),
            log: vec![],
            low_energy_ticks: 0,
            apoptosis_patience: 10,
            actions_taken: 0,
            successful_actions: 0,
        }
    }

    pub fn add_gene(&mut self, gene: Gene) { self.genes.insert(gene.name.clone(), gene); }

    pub fn add_enzyme(&mut self, enzyme: Enzyme) { self.enzymes.push(enzyme); }

    /// Circadian modulation factor for an instinct
    pub fn instinct_modulation(&self, instinct: Instinct) -> f64 {
        let peak = match instinct {
            Instinct::Navigate | Instinct::Play | Instinct::Create => 12.0,
            Instinct::Perceive | Instinct::Communicate | Instinct::Socialize => 14.0,
            Instinct::Rest => 2.0,
            Instinct::Survive | Instinct::Defend => 0.0, // always available
            Instinct::Learn => 10.0,
        };
        let phase = ((self.circadian_hour - peak) / 24.0) * 2.0 * std::f64::consts::PI;
        let raw = (phase.cos() + 1.0) / 2.0;
        0.1 + raw * 0.9 // floor at 0.1
    }

    /// Try to activate an instinct. Returns (success, energy_spent, bytecode_produced)
    pub fn activate_instinct(&mut self, instinct: Instinct, signal: &[u8]) -> (bool, f64, Vec<u8>) {
        if self.dead { return (false, 0.0, vec![]); }

        // Check membrane for dangerous signals
        if !self.membrane.check(signal) {
            self.log.push(format!("MEMBRANE BLOCKED signal for {}", instinct.name()));
            return (false, 0.0, vec![]);
        }

        // Calculate energy cost with circadian modulation
        let base_cost = instinct.energy_cost();
        let modulation = self.instinct_modulation(instinct);

        if base_cost > 0.0 {
            // Consuming energy: cost modulated by circadian
            let cost = base_cost * (0.5 + modulation * 0.5);
            if !self.spend_energy(cost) { return (false, 0.0, vec![]); }
            self.log.push(format!("ACTIVATED {} (cost={:.2}, mod={:.2})", instinct.name(), cost, modulation));
            return (true, cost, vec![]);
        } else {
            // Generating energy (Rest instinct)
            let gen = -base_cost * (0.5 + modulation * 0.5);
            self.energy = (self.energy + gen).min(self.max_energy);
            self.log.push(format!("REST generated {:.2} ATP (mod={:.2})", gen, modulation));
            return (true, 0.0, vec![]);
        }
    }

    /// Find best gene for a signal via enzyme binding
    pub fn find_gene(&self, signal: &[u8]) -> Option<(String, f64)> {
        let mut best: Option<(String, f64)> = None;
        for enzyme in &self.enzymes {
            let strength = enzyme.try_bind(signal);
            if strength > 0.0 {
                for gene_name in &enzyme.target_genes {
                    if let Some(gene) = self.genes.get(gene_name) {
                        let score = strength * gene.fitness * gene.expression;
                        match &best {
                            None => best = Some((gene_name.clone(), score)),
                            Some((_, best_score)) if score > *best_score => best = Some((gene_name.clone(), score)),
                            _ => {}
                        }
                    }
                }
            }
        }
        best
    }

    /// Execute a gene's bytecode (conceptual — returns the action bytes)
    pub fn execute_gene(&mut self, gene_name: &str) -> Option<Vec<u8>> {
        let gene = self.genes.get_mut(gene_name)?;
        if gene.bytecode.is_empty() { return None; }
        gene.use_count += 1;
        let bytecode = gene.bytecode.clone();
        self.actions_taken += 1;
        Some(bytecode)
    }

    /// Record outcome for a gene
    pub fn record_outcome(&mut self, gene_name: &str, success: bool) {
        if let Some(gene) = self.genes.get_mut(gene_name) {
            gene.record_outcome(success);
        }
        if success { self.successful_actions += 1; }
    }

    fn spend_energy(&mut self, amount: f64) -> bool {
        if self.energy < amount {
            self.low_energy_ticks += 1;
            if self.low_energy_ticks >= self.apoptosis_patience {
                self.trigger_apoptosis("Energy depleted");
            }
            return false;
        }
        self.energy -= amount;
        self.low_energy_ticks = self.low_energy_ticks.saturating_sub(1);
        true
    }

    fn trigger_apoptosis(&mut self, reason: &str) {
        self.dead = true;
        self.death_reason = reason.to_string();
        self.log.push(format!("APOPTOSIS: {}", reason));
    }

    pub fn tick(&mut self) {
        if self.dead { return; }
        // Advance circadian clock
        self.circadian_hour = (self.circadian_hour + 0.1) % 24.0;
    }

    /// Quarantine genes that should be auto-quarantined
    pub fn quarantine_bad_genes(&mut self) -> Vec<String> {
        let mut quarantined = vec![];
        let names: Vec<String> = self.genes.keys().cloned().collect();
        for name in names {
            if let Some(gene) = self.genes.get(&name) {
                if gene.should_quarantine() {
                    quarantined.push(name.clone());
                }
            }
        }
        for name in &quarantined {
            self.genes.remove(name);
            self.log.push(format!("QUARANTINED gene: {}", name));
        }
        quarantined
    }

    /// Gene crossover for reproduction
    pub fn crossover_genes(&self, other: &Self, rate: f64) -> Vec<Gene> {
        let mut children = vec![];
        let all_names: Vec<&String> = self.genes.keys().chain(other.genes.keys()).collect();
        for name in all_names {
            let parent1 = self.genes.get(name);
            let parent2 = other.genes.get(name);
            let gene = match (parent1, parent2) {
                (Some(a), Some(b)) if rand() < rate => {
                    let mut child = a.clone();
                    child.bytecode = if rand() < 0.5 { a.bytecode.clone() } else { b.bytecode.clone() };
                    child.fitness = (a.fitness + b.fitness) / 2.0;
                    child.confidence = (a.confidence + b.confidence) / 2.0 * 0.9;
                    child.use_count = 0;
                    child.success_count = 0;
                    child
                }
                (Some(a), _) => a.clone(),
                (_, Some(b)) => b.clone(),
                _ => continue,
            };
            children.push(gene);
        }
        children
    }

    /// Overall fitness score
    pub fn overall_fitness(&self) -> f64 {
        if self.actions_taken == 0 { return 0.5; }
        let action_rate = self.successful_actions as f64 / self.actions_taken as f64;
        let energy_ratio = self.energy / self.max_energy;
        let gene_fitness: f64 = self.genes.values().map(|g| g.fitness).sum::<f64>() / self.genes.len().max(1) as f64;
        (action_rate * 0.4 + energy_ratio * 0.3 + gene_fitness * 0.3).clamp(0.0, 1.0)
    }
}

fn rand() -> f64 {
    // Simple pseudo-random (not crypto-grade, fine for gene simulation)
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    // xorshift
    let mut s = (seed as u64).wrapping_mul(6364136223846793005);
    s ^= s >> 22;
    s = s.wrapping_mul(0x5bd1e995);
    s ^= s >> 15;
    s ^= s >> 27;
    s = s.wrapping_mul(0x5bd1e995);
    (s >> 33) as f64 / u32::MAX as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instinct_energy_costs() {
        assert!(Instinct::Rest.energy_cost() < 0.0); // generates energy
        assert!(Instinct::Perceive.energy_cost() > 0.0);
        assert!(Instinct::Navigate.energy_cost() > 0.0);
        assert!(Instinct::Create.energy_cost() > Instinct::Communicate.energy_cost());
    }

    #[test]
    fn test_gene_quarantine() {
        let mut gene = Gene::new("bad_gene", Instinct::Navigate);
        gene.use_count = 20;
        gene.success_count = 1; // 5% success rate
        gene.fitness = 0.05;
        assert!(gene.should_quarantine());
        gene.success_count = 3; // 15% — at boundary
        assert!(!gene.should_quarantine());
    }

    #[test]
    fn test_enzyme_binding() {
        let enzyme = Enzyme::new("nav_enzyme", vec![1, 0, 1, 0], vec!["navigate_gene"]);
        assert!(enzyme.try_bind(&[1, 0, 1, 0]) > 0.0); // perfect match
        assert!(enzyme.try_bind(&[0, 0, 0, 0]) < 0.6);
        assert_eq!(enzyme.try_bind(&[1, 0]), 0.0); // wrong length
    }

    #[test]
    fn test_membrane_blocking() {
        let m = Membrane::default_antibodies();
        assert!(!m.check(b"sudo rm -rf /")); // blocked
        assert!(m.check(b"read file")); // safe
        assert!(!m.check(b"format disk")); // blocked
    }

    #[test]
    fn test_biological_agent_basic() {
        let mut agent = BiologicalAgent::new("agent-1", 100.0);
        let gene = Gene::new("nav_gene", Instinct::Navigate);
        agent.add_gene(gene);
        let (ok, cost, _) = agent.activate_instinct(Instinct::Navigate, b"some_signal");
        assert!(ok);
        assert!(cost > 0.0);
        assert!(agent.energy < 100.0);
    }

    #[test]
    fn test_rest_generates_energy() {
        let mut agent = BiologicalAgent::new("agent-1", 100.0);
        agent.energy = 50.0;
        let (ok, cost, _) = agent.activate_instinct(Instinct::Rest, b"");
        assert!(ok);
        assert!(agent.energy > 50.0);
    }

    #[test]
    fn test_apoptosis() {
        let mut agent = BiologicalAgent::new("agent-1", 100.0);
        agent.energy = 0.01;
        agent.apoptosis_patience = 3;
        agent.activate_instinct(Instinct::Navigate, b"sig");
        agent.activate_instinct(Instinct::Navigate, b"sig");
        assert!(!agent.dead);
        agent.activate_instinct(Instinct::Navigate, b"sig");
        assert!(agent.dead);
        assert!(agent.death_reason.contains("Energy"));
    }

    #[test]
    fn test_circadian_modulation() {
        let mut agent = BiologicalAgent::new("agent-1", 100.0);
        let noon = agent.instinct_modulation(Instinct::Navigate);
        agent.circadian_hour = 0.0;
        let midnight = agent.instinct_modulation(Instinct::Navigate);
        assert!(noon > midnight);
    }

    #[test]
    fn test_gene_outcome() {
        let mut gene = Gene::new("test", Instinct::Perceive);
        gene.record_outcome(true);
        gene.record_outcome(true);
        gene.record_outcome(false);
        assert!((gene.success_rate() - 0.666).abs() < 0.01);
        assert!(gene.fitness > 0.5);
    }

    #[test]
    fn test_rna_translation() {
        let mut gene = Gene::new("test", Instinct::Navigate);
        gene.bytecode = vec![0x03, 0x00, 0x10, 0x00];
        let rna = RnaMessenger::translate(&gene);
        assert_eq!(rna.translated_bytecode, gene.bytecode);
        assert_eq!(rna.source_gene, "test");
    }

    #[test]
    fn test_quarantine_in_agent() {
        let mut agent = BiologicalAgent::new("agent-1", 100.0);
        let mut bad = Gene::new("bad", Instinct::Navigate);
        bad.use_count = 20; bad.success_count = 1; bad.fitness = 0.05;
        agent.add_gene(bad);
        let q = agent.quarantine_bad_genes();
        assert_eq!(q.len(), 1);
        assert!(agent.genes.get("bad").is_none());
    }

    #[test]
    fn test_gene_crossover() {
        let mut a = BiologicalAgent::new("a", 100.0);
        let mut b = BiologicalAgent::new("b", 100.0);
        let mut g1 = Gene::new("shared", Instinct::Navigate);
        g1.bytecode = vec![1, 2, 3];
        g1.fitness = 0.8;
        let mut g2 = Gene::new("shared", Instinct::Navigate);
        g2.bytecode = vec![4, 5, 6];
        g2.fitness = 0.6;
        a.add_gene(g1);
        b.add_gene(g2);
        let children = a.crossover_genes(&b, 1.0);
        assert!(!children.is_empty());
    }

    #[test]
    fn test_overall_fitness() {
        let agent = BiologicalAgent::new("agent-1", 100.0);
        let f = agent.overall_fitness();
        assert!(f >= 0.0 && f <= 1.0);
    }

    #[test]
    fn test_tick_circadian() {
        let mut agent = BiologicalAgent::new("agent-1", 100.0);
        let h1 = agent.circadian_hour;
        agent.tick();
        let h2 = agent.circadian_hour;
        assert!(h2 > h1);
    }

    #[test]
    fn test_find_gene() {
        let mut agent = BiologicalAgent::new("agent-1", 100.0);
        let gene = Gene::new("nav", Instinct::Navigate);
        agent.add_gene(gene);
        let enzyme = Enzyme::new("nav_enz", vec![1, 0], vec!["nav"]);
        agent.add_enzyme(enzyme);
        let found = agent.find_gene(&[1, 0]);
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, "nav");
    }
}
