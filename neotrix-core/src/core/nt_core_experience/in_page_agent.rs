// G398 + G407: In-page JS agent engine — Page-Agent inspired, text-based DOM manipulation
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomAction {
    Click { selector: String },
    Fill { selector: String, value: String },
    Select { selector: String, value: String },
    Hover { selector: String },
    Scroll { x: i32, y: i32 },
    Wait { ms: u64 },
    Extract { selector: String, attribute: String },
    Evaluate { script: String },
    Navigate { url: String },
    Submit { selector: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomElement {
    pub tag: String,
    pub selector: String,
    pub text: String,
    pub attributes: Vec<(String, String)>,
    pub is_visible: bool,
    pub is_interactive: bool,
    pub bounding_box: Option<[f64; 4]>,
    pub children: Vec<DomElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    pub steps: Vec<DomAction>,
    pub description: String,
    pub required_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Success {
        output: String,
        dom_snapshot: String,
    },
    Failure {
        reason: String,
        dom_state: String,
    },
    Timeout {
        partial_output: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentGoal {
    FillForm,
    ClickButton,
    ExtractData,
    NavigateFlow,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InPageAgent {
    pub current_url: String,
    pub page_title: String,
    pub dom_tree: Vec<DomElement>,
    pub action_history: Vec<(DomAction, ActionResult)>,
    pub max_actions: usize,
    pub max_depth: usize,
}

impl InPageAgent {
    pub fn new() -> Self {
        Self {
            current_url: String::new(),
            page_title: String::new(),
            dom_tree: Vec::new(),
            action_history: Vec::new(),
            max_actions: 100,
            max_depth: 5,
        }
    }

    pub fn set_page(&mut self, url: &str, title: &str, dom: Vec<DomElement>) {
        self.current_url = url.to_string();
        self.page_title = title.to_string();
        self.dom_tree = dom;
    }

    pub fn plan_action(&self, goal: &AgentGoal) -> ActionPlan {
        match goal {
            AgentGoal::FillForm => self.plan_fill_form(),
            AgentGoal::ClickButton => self.plan_click_button(),
            AgentGoal::ExtractData => self.plan_extract_data(),
            AgentGoal::NavigateFlow => self.plan_navigate_flow(),
            AgentGoal::Custom(desc) => ActionPlan {
                steps: vec![],
                description: desc.clone(),
                required_confidence: 0.5,
            },
        }
    }

    fn plan_fill_form(&self) -> ActionPlan {
        let mut steps = Vec::new();
        for element in &self.dom_tree {
            let tag = element.tag.to_lowercase();
            if tag == "input" || tag == "textarea" || tag == "select" {
                let action = if tag == "select" {
                    DomAction::Select {
                        selector: element.selector.clone(),
                        value: String::new(),
                    }
                } else {
                    DomAction::Fill {
                        selector: element.selector.clone(),
                        value: String::new(),
                    }
                };
                steps.push(action);
            }
        }
        let n = steps.len();
        ActionPlan {
            steps,
            description: format!("Fill {} form fields", n),
            required_confidence: 0.7,
        }
    }

    fn plan_click_button(&self) -> ActionPlan {
        let mut steps = Vec::new();
        for element in &self.dom_tree {
            let tag = element.tag.to_lowercase();
            if tag == "button"
                || (tag == "a" && element.attributes.iter().any(|(k, _)| k == "href"))
            {
                steps.push(DomAction::Click {
                    selector: element.selector.clone(),
                });
            }
        }
        let n = steps.len().min(5);
        ActionPlan {
            steps: steps.into_iter().take(5).collect(),
            description: format!("Click {} interactive elements", n),
            required_confidence: 0.8,
        }
    }

    fn plan_extract_data(&self) -> ActionPlan {
        let mut steps = Vec::new();
        for element in &self.dom_tree {
            let tag = element.tag.to_lowercase();
            if matches!(
                tag.as_str(),
                "p" | "h1" | "h2" | "h3" | "td" | "li" | "span"
            ) {
                steps.push(DomAction::Extract {
                    selector: element.selector.clone(),
                    attribute: "text".to_string(),
                });
            }
        }
        let n = steps.len().min(20);
        ActionPlan {
            steps: steps.into_iter().take(20).collect(),
            description: format!("Extract text from {} elements", n),
            required_confidence: 0.6,
        }
    }

    fn plan_navigate_flow(&self) -> ActionPlan {
        let steps = vec![
            DomAction::Wait { ms: 500 },
            DomAction::Evaluate {
                script: "document.readyState".to_string(),
            },
        ];
        ActionPlan {
            steps,
            description: "Check page load state and prepare navigation".to_string(),
            required_confidence: 0.5,
        }
    }

    pub fn record_action(&mut self, action: DomAction, result: ActionResult) {
        if self.action_history.len() >= self.max_actions {
            self.action_history.remove(0);
        }
        self.action_history.push((action, result));
    }

    pub fn find_element_by_text(&self, text: &str) -> Vec<&DomElement> {
        let mut found = Vec::new();
        self.collect_by_text(&self.dom_tree, text, &mut found, 0);
        found
    }

    fn collect_by_text<'a>(
        &'a self,
        elements: &'a [DomElement],
        text: &str,
        acc: &mut Vec<&'a DomElement>,
        depth: usize,
    ) {
        if depth > self.max_depth {
            return;
        }
        for element in elements {
            if element.text.contains(text) {
                acc.push(element);
            }
            self.collect_by_text(&element.children, text, acc, depth + 1);
        }
    }

    pub fn find_interactive_elements(&self) -> Vec<&DomElement> {
        let mut interactive = Vec::new();
        self.collect_interactive(&self.dom_tree, &mut interactive, 0);
        interactive
    }

    fn collect_interactive<'a>(
        &'a self,
        elements: &'a [DomElement],
        acc: &mut Vec<&'a DomElement>,
        depth: usize,
    ) {
        if depth > self.max_depth {
            return;
        }
        for element in elements {
            if element.is_interactive && element.is_visible {
                acc.push(element);
            }
            self.collect_interactive(&element.children, acc, depth + 1);
        }
    }
}
