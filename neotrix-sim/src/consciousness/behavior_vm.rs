use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    Move { dx: f32, dy: f32 },
    Harvest { resource: String },
    Trade { target: String, offer: String, want: String },
    Talk { target: String, message: String },
    Build { structure: String },
    Attack { target: String },
    Rest,
    Think { topic: String },
    If { condition: Condition, then_box: Box<Instruction>, else_box: Option<Box<Instruction>> },
    Repeat { times: u32, body: Box<Instruction> },
    Sequence(Vec<Instruction>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    HealthBelow(f32),
    EnergyBelow(f32),
    HasResource(String),
    NearAgent(String),
    HasStructure(String),
    TimeOfDay(String),
    True,
}

pub struct BehaviorVm {
    program: Vec<Instruction>,
    pc: usize,  // program counter
    stack_depth: u32,
    max_stack: u32,
    completed_cycles: u64,
}

impl BehaviorVm {
    pub fn new(max_stack: u32) -> Self {
        Self { program: Vec::new(), pc: 0, stack_depth: 0, max_stack, completed_cycles: 0 }
    }

    pub fn load(&mut self, program: Vec<Instruction>) {
        self.program = program;
        self.pc = 0;
    }

    pub fn load_default_survival(&mut self) {
        self.load(vec![
            Instruction::If {
                condition: Condition::HealthBelow(30.0),
                then_box: Box::new(Instruction::Rest),
                else_box: Some(Box::new(Instruction::Sequence(vec![
                    Instruction::If {
                        condition: Condition::EnergyBelow(20.0),
                        then_box: Box::new(Instruction::Rest),
                        else_box: Some(Box::new(Instruction::Sequence(vec![
                            Instruction::Think { topic: "explore".into() },
                            Instruction::Repeat {
                                times: 5,
                                body: Box::new(Instruction::Sequence(vec![
                                    Instruction::Move { dx: 10.0, dy: 0.0 },
                                    Instruction::Harvest { resource: "food".into() },
                                ])),
                            },
                        ]))),
                    },
                ]))),
            },
        ]);
    }

    pub fn step(&mut self, condition_eval: &dyn Fn(&Condition) -> bool) -> Option<Instruction> {
        if self.pc >= self.program.len() {
            self.completed_cycles += 1;
            self.pc = 0;
            if self.program.is_empty() { return None; }
        }

        let instruction = self.program.remove(self.pc);

        match &instruction {
            Instruction::If { condition, then_box, else_box } => {
                if condition_eval(condition) {
                    self.stack_depth += 1;
                    if self.stack_depth > self.max_stack { return None; }
                    self.program.insert(self.pc, *then_box.clone());
                } else if let Some(else_inst) = else_box {
                    self.program.insert(self.pc, *else_inst.clone());
                }
                self.step(condition_eval)
            }
            Instruction::Repeat { times, body } => {
                for i in 0..*times {
                    self.program.insert(self.pc + i as usize, *body.clone());
                }
                self.step(condition_eval)
            }
            Instruction::Sequence(items) => {
                for (i, item) in items.iter().enumerate() {
                    self.program.insert(self.pc + i, item.clone());
                }
                self.step(condition_eval)
            }
            _ => Some(instruction),
        }
    }

    pub fn reset(&mut self) { self.pc = 0; self.stack_depth = 0; }
    pub fn is_finished(&self) -> bool { self.pc >= self.program.len() && self.stack_depth == 0 }
    pub fn completed_cycles(&self) -> u64 { self.completed_cycles }
    pub fn program_len(&self) -> usize { self.program.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_and_step() {
        let mut vm = BehaviorVm::new(10);
        vm.load(vec![Instruction::Rest, Instruction::Think { topic: "test".into() }]);
        let eval = |_: &Condition| false;
        let i = vm.step(&eval).unwrap();
        assert!(matches!(i, Instruction::Rest));
        let i = vm.step(&eval).unwrap();
        assert!(matches!(i, Instruction::Think { .. }));
    }

    #[test]
    fn if_condition_true() {
        let mut vm = BehaviorVm::new(10);
        vm.load(vec![Instruction::If {
            condition: Condition::HealthBelow(50.0),
            then_box: Box::new(Instruction::Rest),
            else_box: None,
        }]);
        let eval = |c: &Condition| matches!(c, Condition::HealthBelow(t) if *t == 50.0);
        let i = vm.step(&eval).unwrap();
        assert!(matches!(i, Instruction::Rest));
    }

    #[test]
    fn repeat_unrolls() {
        let mut vm = BehaviorVm::new(10);
        vm.load(vec![Instruction::Repeat { times: 3, body: Box::new(Instruction::Rest) }]);
        let eval = |_: &Condition| false;
        let mut count = 0;
        while let Some(_) = vm.step(&eval) { count += 1; }
        assert_eq!(count, 3);
    }

    #[test]
    fn empty_program_returns_none() {
        let mut vm = BehaviorVm::new(10);
        let eval = |_: &Condition| false;
        assert!(vm.step(&eval).is_none());
    }

    #[test]
    fn default_survival_loads() {
        let mut vm = BehaviorVm::new(10);
        vm.load_default_survival();
        assert!(vm.program_len() > 0);
    }
}
