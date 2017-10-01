

struct Rule {
    alternatives: Box<[Alternative]>,
}


struct Alternative {
    terms: Box<[Term]>,
}

enum Term {
    Terminal { value: Box<str> },
    Rule { key: RuleKey },
}

struct PatternMatcher {
    rule: &Rule,
    input: &str,
    variant: usize,
    children: Vec<PatternMatcher>,
}

impl PatternMatcher {
    fn new(rule: &Rule, input: &str) -> Self {
        PatternMatcher {
            rule: rule,
            input: input,
            variant: ~0,
            children: Vec::new(),
        }
    }

    fn is_unfinished(&self) -> bool {
        self.variant + 1 <= self.rule.alternatives.len()
    }

    fn needs_rewind(&self) -> bool {
        !self.children.is_empty() && self.children.last().is_unfinished()
    }

    fn next_child(&self) -> Result<(RuleKey, &str), bool> {
        let mut leftover = self.children.last().leftover();
        let mut curr_term = self.curr_term();
        let terms = self.rule.alternatives[self.variant].terms;
        let (_, terms_left) = terms.split(curr_term);
        for term in terms_left {
            match term {
                Terminal { ref value } => {
                    if leftover.starts_with(value) {
                        (_, leftover) = {leftover}.split_at(value.len());
                        curr_term += 1;
                    } else {
                        return Err(false);
                    }
                },
                Rule { key } => {
                    return Ok((key, leftover));
                },
            }
        }
        Err(true)
    }

    fn find_next(&mut self) {
        self.children.last_mut().find_next();
        while self.variant < self.rule.variants.len() {
            if self.needs_rewind() {
                self.children.pop();
                self.children.last_mut().find_next();
            } else if !self.children.is_empty() {
                match self.next_child() {
                    // terminals matched, nonterminal found
                    Ok(key, leftover) => {
                        let sub_rule = &rules[key];
                        let next_match = PatternMatcher::new(sub_rule, leftover);
                        self.children.push(next_match);
                    },
                    // terminals matched, rule satisfied
                    Err(true) => {
                        break;
                    },
                    // terminals not matched
                    Err(false) => {
                        self.children.last_mut().find_next();
                    },
                }
            } else {
                self.variant += 1;
            }
        }
    }
}



type RuleKey = i32;


fn terminal(input: &str) -> Term {
    let owned = String::from(input);
    Term::Terminal {
        value: owned.as_boxed_str(),
    }
}

fn terminal_alternative(input: &str) -> Alternative {
    Alternative{
        terms: Box::<[Term; 1]>::new(terminal(input)),
    }
}

#[cfg(test)]
mod tests {
    const FUNPAR: RuleKey = 0;
    const FUNID: RuleKey = 1;
    const PARID: RuleKey = 2;

    fn make_rules() -> Rules {
        let funpar = Alternative {
            terms: Box::<[Term; 6]>::new([
                Term::Rule { key: FUNID },
                terminal("("),
                Term::Rule { key: PARID },
                terminal(", "),
                Term::Rule { key: PARID },
                terminal(")"),
            ]),
        };
        let funpar = Rule {
            alternatives: Box::<[Alternative; 1]>::new([funpar]),
        };

        let funid = [terminal_alternative("f"), terminal_alternative("g")];
        let funid = Rule {
            alternatives: Box::<[Alternative; 2]>::new(funid),
        };

        let parid = [terminal_alternative("x"), terminal_alternative("y")];
        let parid = Rule {
            alternatives: Box::<[Alternative; 2]>::new(parid),
        };

        vec![funpar, funid, parid]
    }

    #[test]
    fn it_works() {
        let input = "f(x, y)";
        let rules = make_rules();
        let mut matcher = PatternMatcher::new(&rules, FUNPAR, input);
        matcher.find_next();
        assert_eq!(matcher.pattern(), pattern(vec![0,0,1]));
    }
}
