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

enum MatcherState<'s> {
    Valid,
    Rewind,
    IterateLast,
    AppendMatcher(RuleKey, &'s str),
}

pub struct PatternMatcher<'r, 's> {
    rules: &'r Rules,
    rule: &'r Rule,
    input: &'s str,
    variant: usize,
    children: Vec<PatternMatcher<'r, 's>>,
}

#[derive(Clone)]
pub struct Pattern {
    pub variant: usize,
    pub children: Box<[Rc<Pattern>]>,
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
        self.variant.wrapping_add(1) <= self.rule.alternatives.len()
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
                .indices[self.children.len() - 1] + 1
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

    fn iteration_state(&self) -> MatcherState<'s> {
        if self.needs_rewind() {
            return MatcherState::Rewind;
        }
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
                        return MatcherState::IterateLast;
                    }
                },
                &Term::Rule { key } => {
                    return MatcherState::AppendMatcher(key, leftover);
                },
            }
        }
        MatcherState::Valid
    }

    // call a non-naive find next on the last child if any,
    // if there is no child move on to the next variant instead
    fn find_next_naive(&mut self) {
        if let Some(child) = self.children.last_mut() {
            child.find_next()
        } else {
            self.variant = self.variant.wrapping_add(1);
        }
    }

    pub fn find_next(&mut self) {
        self.find_next_naive();
        while self.is_unfinished() {
            match self.iteration_state() {
                MatcherState::Rewind => {
                    self.children.pop();
                    self.find_next_naive();
                },
                // terminals matched, nonterminal found
                MatcherState::AppendMatcher(key, leftover) => {
                    let mut next_match = PatternMatcher::new(self.rules, key, leftover);
                    next_match.find_next();
                    self.children.push(next_match);
                },
                // terminals matched, rule satisfied
                MatcherState::Valid => {
                    break;
                },
                // terminals not matched
                MatcherState::IterateLast => {
                    self.find_next_naive();
                },
            }
        }
    }

    pub fn pattern(&self) -> Pattern
        let children: Vec<Rc<Pattern>> =
            self.children
                .iter()
                .map(|child| Rc::new(child.pattern()))
                .collect();
        PatternInt{
            variant: self.variant,
            children: children.into_boxed_slice(),
        }

    }

    pub fn pattern_with(&self, patt: &mut Vec<usize>) {
        if self.variant != 0 || self.rule.alternatives.len() > 1 {
            patt.push(self.variant);
        }
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
        assert_eq!(matcher.pattern(), vec![usize::max_value()]);
        matcher.find_next();
        assert_eq!(matcher.pattern(), vec![0,0,1]);
    }
}
