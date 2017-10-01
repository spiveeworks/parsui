

pub struct Rule {
    pub alternatives: Box<[Alternative]>,
}


pub struct Alternative {
    terms: Box<[Term]>,
    indices: Box<[usize]>,
}

impl Alternative {
    pub fn new(terms: Box<[Term]>) -> Self {
        let mut indices = Vec::with_capacity(terms.len() + 1);

        for (index, term) in terms.iter().enumerate() {
            if let &Term::Rule{..} = term {
                indices.push(index);
            }
        }

        Alternative {
            terms: terms,
            indices: indices.into_boxed_slice(),
        }
    }
}


pub enum Term {
    Terminal { value: Box<str> },
    Rule { key: RuleKey },
}

pub struct PatternMatcher<'r, 's> {
    rules: &'r Rules,
    rule: &'r Rule,
    input: &'s str,
    variant: usize,
    children: Vec<PatternMatcher<'r, 's>>,
}

impl<'r, 's> PatternMatcher<'r, 's> {
    pub fn new(rules: &'r Rules, rule: RuleKey, input: &'s str) -> Self {
        PatternMatcher {
            rules: rules,
            rule: &rules[rule],
            input: input,
            variant: usize::max_value(),
            children: Vec::new(),
        }
    }

    fn is_unfinished(&self) -> bool {
        self.variant + 1 <= self.rule.alternatives.len()
    }

    fn needs_rewind(&self) -> bool {
        self.children
            .last()
            .map(|child| !child.is_unfinished()) // do need to unwind if child ran out of possibilities
            .unwrap_or(false) // don't need to unwind if there are no children
    }

    fn alternative(&self) -> &Alternative {
        &self.rule.alternatives[self.variant]
    }

    fn curr_term(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            self.alternative()
                .indices[self.children.len()]
        }
    }

    fn terms_left(&self) -> &[Term] {
        self.alternative()
            .terms
            .split_at(self.curr_term())
            .1
    }

    fn leftover(&self) -> &'s str {
        let mut result = self.input;
        for term in self.terms_left() {
            if let &Term::Terminal { ref value } = term {
                result = {result}.split_at(value.len()).1;
            }
        }
        result
    }

    fn next_child(&self) -> Result<(RuleKey, &'s str), bool> {
        let mut leftover = if let Some(child) = self.children.last() {
            child.leftover()
        } else {
            self.input
        };
        for term in self.terms_left() {
            match term {
                &Term::Terminal { value: ref expected } => {
                    let (actual, new_leftover) = {leftover}.split_at(expected.len());
                    if **expected == *actual {
                        leftover = new_leftover;
                    } else {
                        return Err(false);
                    }
                },
                &Term::Rule { key } => {
                    return Ok((key, leftover));
                },
            }
        }
        Err(true)
    }

    pub fn find_next(&mut self) {
        if let Some(child) = self.children.last_mut() {
            child.find_next()
        } else {
            self.variant += 1;
        }
        while self.variant < self.rule.alternatives.len() {
            if self.needs_rewind() {
                self.children.pop();
                self.children.last_mut().unwrap().find_next();
            } else if !self.children.is_empty() {
                match self.next_child() {
                    // terminals matched, nonterminal found
                    Ok((key, leftover)) => {
                        let next_match = PatternMatcher::new(self.rules, key, leftover);
                        self.children.push(next_match);
                    },
                    // terminals matched, rule satisfied
                    Err(true) => {
                        break;
                    },
                    // terminals not matched
                    Err(false) => {
                        self.children.last_mut().unwrap().find_next();
                    },
                }
            } else {
                self.variant += 1;
            }
        }
    }

    pub fn pattern(&self) -> Vec<usize> {
        let mut result = Vec::new();
        self.pattern_with(&mut result);
        result
    }

    pub fn pattern_with(&self, patt: &mut Vec<usize>) {
        patt.push(self.variant);
        for child in &*self.children {
            child.pattern_with(patt);
        }
    }
}



type RuleKey = usize;
type Rules = Vec<Rule>;



#[cfg(test)]
mod tests {

    use super::*;

    const FUNPAR: RuleKey = 0;
    const FUNID: RuleKey = 1;
    const PARID: RuleKey = 2;

    fn terminal<'a>(input: &'a str) -> Term {
        let owned = String::from(input);
        Term::Terminal {
            value: owned.into_boxed_str(),
        }
    }

    fn terminal_alternative<'a>(input: &'a str) -> Alternative {
        let term: Box<[Term; 1]> = Box::new([terminal(input)]);
        Alternative::new(term)
    }

    fn make_rules() -> Rules {
        let funpar = Alternative::new(vec![
            Term::Rule { key: FUNID },
            terminal("("),
            Term::Rule { key: PARID },
            terminal(", "),
            Term::Rule { key: PARID },
            terminal(")"),
        ].into_boxed_slice());
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
        assert_eq!(matcher.pattern(), vec![0,0,1]);
    }
}
