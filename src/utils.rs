pub struct IsClosed {
    pub in_arrow: bool,
    pub in_cond: bool,
    recent: Vec<TOCLOSE>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TOCLOSE {
    BRACKETS,
    PARENS,
    QUOTES,
}

impl IsClosed {
    pub fn new() -> IsClosed {
        IsClosed {
            recent: Vec::new(),
            in_arrow: false,
            in_cond: false,
        }
    }

    pub fn check(&mut self, sym: &str) {
        let a;
        if self.recent.len() == 0 {
            a = true;
        } else {
            a = self.recent[self.recent.len() - 1] != TOCLOSE::QUOTES;
        };
        if a {
            match sym {
                "(" => {
                    self.recent.push(TOCLOSE::BRACKETS);
                }
                "{" => {
                    self.recent.push(TOCLOSE::PARENS);
                }
                "}" => {
                    if self.recent[self.recent.len() - 1] == TOCLOSE::PARENS {
                        self.recent.pop();
                    } else {
                        panic!("close {:?} first", self.recent[self.recent.len() - 1]);
                    }
                }
                ")" => {
                    if self.recent[self.recent.len() - 1] == TOCLOSE::BRACKETS {
                        self.recent.pop();
                    } else {
                        panic!("close {:?} first", self.recent[self.recent.len() - 1]);
                    }
                }
                "\"" => {
                    self.recent.push(TOCLOSE::QUOTES);
                }

                _ => {}
            }
        } else if sym == "\"" {
            self.recent.pop();
        }
    }

    pub fn is(&self) -> bool {
        self.recent.len() == 0
    }

    pub fn unclosed(&self) -> TOCLOSE {
        self.recent[self.recent.len() - 1]
    }
}
