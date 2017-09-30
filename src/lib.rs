

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

struct Pattern {
    variant: usize,
    content: Box<[Pattern]>,
}

struct PatternMatcher {
    rule: &Rule,
    input: &str,
    variant: usize,
    children: Box<[PatternMatcher]>,
}

impl PatternMatcher {
    fn new(rule: &Rule, input: &str) -> Self {
        let mut result = PatternMatcher {
            rule: rule,
            input: input,
            variant: 0,
            children: Box::<[_; 0]>::new([]),
        };
        let sub_matches = Vec::new();

        while result.variant < rule.alternatives.len() {
        {
            let alternative = &rule.alternatives[result.variant];
            sub_matches.empty();
            let mut leftover = input;
            let mut curr_term = 0;
            while curr_term < alternative.terms.len() {
                match alternative.terms[curr_term] {
                    Terminal { ref value } => {
                        while !leftover.starts_with(value) {
                            sub_matches.last_mut().next();
                            // break if iterating the last sub_match failed
                            leftover = sub_matches.last().leftover;
                        }
                        (_, leftover) = {leftover}.split_at(value.len());
                    },
                    Rule { key } => {
                        let sub_rule = &rules[key];
                        let next_match = PatternMatcher::new(sub_rule, leftover);
                        sub_matches.push(next_match);
                    },
                }
                curr_term += 1; // but what if matching a terminal fails?
                while !sub_matches.is_empty() && !sub_matches.last().failed() {
                    sub_matches.pop();
                    term -= 1;
                    while let Terminal{..} = alternative.terms[curr_term] {
                        term -= 1;
                    }
                    sub_matches.last_mut().next();
                }
                if curr_term == 0 {
                    break;
                }
            }
            if curr_term == alternative.terms.len() {
                result.children = sub_matches.into_boxed_slice();
                return result;
            }
            result.variant += 1;
        }
        result
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
