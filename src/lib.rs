

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




fn match<'r, 's, Rs: Index<RuleKey, Output=Rule>>(
    rules: &'r Rs,
    rule: RuleKey,
    input: &'s str)
{
    let rule = &rules[rule];
    for option in rule.alternatives.iter() {
        for



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
