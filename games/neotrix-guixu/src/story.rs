//! 剧情事件脚本 — jynew「游戏事件脚本」提炼（Lua/蓝图→Rust 数据）.
//!
//! 公理：事件 = 有编号的命令序列；命令分五族（显示/流程/数值/场景/音乐）；
//! 触发器三槽（交互/用道具/进入）+ ModifyEvent 运行时改线。
//! 本实现（v1，动作游戏适配）：Say/Choice/Banner/Do/End；
//! 改线体现为 Effect::AddElite（选项改写后续波次 composition）。
//! 纯逻辑；调用方每帧推 advance/choose，取 outbox 副作用执行。

/// 主循环执行的副作用（解释器只生产，主循环消费）
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    GiveQi(f32),
    Heal(f32),
    AddElite(u32),
    Score(u32),
}

/// 选项：文本 + 跳转 + 即时效果
#[derive(Debug, Clone)]
pub struct ChoiceOpt {
    pub text: String,
    pub goto: usize,
    pub effects: Vec<Effect>,
}

/// 命令（jynew 指令表的动作子集）
#[derive(Debug, Clone)]
pub enum Cmd {
    /// 说话（who 说 text），需确认
    Say { who: String, text: String },
    /// 分支（prompt + 选项），需选择
    Choice { prompt: String, options: Vec<ChoiceOpt> },
    /// 即时数值（无需停留）
    Do(Vec<Effect>),
    /// 终幕
    End,
}

#[derive(Debug, Clone)]
pub struct Story {
    cmds: Vec<Cmd>,
    pc: usize,
    pub done: bool,
    /// 待确认的行（speaker, text），None 则无
    pub line: Option<(String, String)>,
    /// 待选项（prompt, texts），None 则无
    pub choices: Option<(String, Vec<String>)>,
    /// 待主循环执行的副作用
    pub outbox: Vec<Effect>,
}

impl Story {
    pub fn new(cmds: Vec<Cmd>) -> Self {
        let mut s = Self {
            cmds,
            pc: 0,
            done: false,
            line: None,
            choices: None,
            outbox: Vec::new(),
        };
        s.present();
        s
    }

    /// 确认当前行 → 推进（无行时空转）
    pub fn advance(&mut self) {
        if self.line.is_some() {
            self.line = None;
            self.present();
        }
    }

    /// 选择 idx（非法索引忽略，不推进）
    pub fn choose(&mut self, idx: usize) {
        let (goto, effects) = match &self.choices {
            Some((_, texts)) if idx < texts.len() => {
                match self.cmds.get(self.pc) {
                    Some(Cmd::Choice { options, .. }) => match options.get(idx) {
                        Some(o) => (o.goto, o.effects.clone()),
                        None => return,
                    },
                    _ => return,
                }
            }
            _ => return,
        };
        self.choices = None;
        self.outbox.extend(effects);
        self.pc = goto;
        self.present();
    }

    /// 执行自动命令直到 Say/Choice/End（越界即终幕，防 panic）
    fn present(&mut self) {
        loop {
            let cmd = match self.cmds.get(self.pc) {
                Some(c) => c.clone(),
                None => {
                    self.done = true;
                    return;
                }
            };
            match cmd {
                Cmd::Say { who, text } => {
                    self.line = Some((who, text));
                    self.pc += 1;
                    return;
                }
                Cmd::Choice { prompt, options } => {
                    self.choices = Some((prompt, options.iter().map(|o| o.text.clone()).collect()));
                    return;
                }
                Cmd::Do(fx) => {
                    self.outbox.extend(fx);
                    self.pc += 1;
                }
                Cmd::End => {
                    self.done = true;
                    self.line = None;
                    self.choices = None;
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear() -> Story {
        Story::new(vec![
            Cmd::Say { who: "说书人".into(), text: "话说南宋绍兴年间。".into() },
            Cmd::Do(vec![Effect::GiveQi(10.0)]),
            Cmd::Say { who: "少侠".into(), text: "剑来。".into() },
            Cmd::End,
        ])
    }

    #[test]
    fn linear_flow_with_auto_effects() {
        let mut s = linear();
        assert!(!s.done);
        assert_eq!(s.line.as_ref().map(|l| l.0.as_str()), Some("说书人"));
        s.advance();
        // Do 自动执行不停留
        assert_eq!(s.outbox, vec![Effect::GiveQi(10.0)]);
        assert_eq!(s.line.as_ref().map(|l| l.0.as_str()), Some("少侠"));
        s.advance();
        assert!(s.done);
        assert!(s.line.is_none());
    }

    #[test]
    fn choice_jumps_and_applies() {
        let mut s = Story::new(vec![
            Cmd::Choice {
                prompt: "夜宿破庙，如何？".into(),
                options: vec![
                    ChoiceOpt { text: "打坐".into(), goto: 1, effects: vec![Effect::Heal(40.0)] },
                    ChoiceOpt { text: "练剑".into(), goto: 2, effects: vec![Effect::AddElite(1)] },
                ],
            },
            Cmd::Say { who: "你".into(), text: "气息绵长。".into() },
            Cmd::Say { who: "你".into(), text: "剑气又利三分。".into() },
            Cmd::End,
        ]);
        assert!(s.choices.is_some());
        s.choose(9); // 非法索引：忽略
        assert!(s.choices.is_some());
        s.choose(1);
        assert_eq!(s.outbox, vec![Effect::AddElite(1)]);
        assert_eq!(s.line.as_ref().map(|l| l.1.as_str()), Some("剑气又利三分。"));
        s.advance();
        assert!(s.done);
    }

    #[test]
    fn empty_and_oob_end_safely() {
        let s = Story::new(vec![]);
        assert!(s.done);
        let mut t = Story::new(vec![Cmd::Say { who: "甲".into(), text: "嗯。".into() }]);
        t.advance();
        assert!(t.done); // 落出末尾即终幕，不 panic
    }
}
