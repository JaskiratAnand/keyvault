use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Separator {
    Space,
    Hyphen,
    Underscore,
    Period,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Capitalization {
    Lowercase,
    Uppercase,
    TitleCase,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum GeneratorConfig {
    Character {
        length: usize,
        min_uppercase: usize,
        min_lowercase: usize,
        min_numbers: usize,
        min_symbols: usize,
        exclude_ambiguous: bool,
    },
    Passphrase {
        words: usize,
        separator: Separator,
        capitalization: Capitalization,
    },
    Pin {
        length: usize,
    },
}

// embed raw EFF wordlist at compile-time
const WORDLIST_DATA: &str = include_str!("../resources/eff_wordlist.txt");
static WORDLIST: OnceLock<Vec<&'static str>> = OnceLock::new();

/// Returns a reference to the parsed static wordlist.
/// Lazily splits the raw file by lines and extracts the word part on the first call.
fn get_wordlist() -> &'static [&'static str] {
    WORDLIST.get_or_init(|| {
        WORDLIST_DATA
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect()
    })
}

/// Generates - password, passphrase, or PIN
pub fn generate(config: &GeneratorConfig) -> Result<(String, f64), &'static str> {
    let mut rng = rand::rng();

    match config {
        GeneratorConfig::Pin { length } => {
            if *length == 0 {
                return Err("Length must be greater than 0");
            }
            let digits = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
            let mut pin = String::new();
            for _ in 0..*length {
                let digit = digits.choose(&mut rng).ok_or("Empty digits pool")?;
                pin.push(*digit);
            }
            let entropy = (*length as f64) * 10.0f64.log2();
            Ok((pin, entropy))
        }

        GeneratorConfig::Passphrase {
            words,
            separator,
            capitalization,
        } => {
            if *words == 0 {
                return Err("Word count must be greater than 0");
            }

            let wordlist = get_wordlist();
            if wordlist.is_empty() {
                return Err("Wordlist is empty");
            }

            let mut selected_words = Vec::new();
            for _ in 0..*words {
                let raw_word = wordlist.choose(&mut rng).ok_or("Empty wordlist")?;

                // Handle capitalization
                let processed_word = match capitalization {
                    Capitalization::Lowercase => raw_word.to_string(),
                    Capitalization::Uppercase => raw_word.to_uppercase(),
                    Capitalization::TitleCase => {
                        let mut chars = raw_word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    }
                };

                selected_words.push(processed_word);
            }

            // Handle separator
            let sep_str = match separator {
                Separator::Space => " ",
                Separator::Hyphen => "-",
                Separator::Underscore => "_",
                Separator::Period => ".",
                Separator::None => "",
            };

            let passphrase = selected_words.join(sep_str);
            let entropy = (*words as f64) * (wordlist.len() as f64).log2();
            Ok((passphrase, entropy))
        }

        GeneratorConfig::Character {
            length,
            min_uppercase,
            min_lowercase,
            min_numbers,
            min_symbols,
            exclude_ambiguous,
        } => {
            let total_min = min_uppercase + min_lowercase + min_numbers + min_symbols;
            if *length < total_min {
                return Err("Requested length is smaller than the sum of minimum requirements");
            }
            if *length == 0 {
                return Err("Length must be greater than 0");
            }

            // Define character pools
            let mut uppercase_pool = vec![
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
                'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            ];
            let mut lowercase_pool = vec![
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            ];
            let mut numbers_pool = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
            let mut symbols_pool = vec![
                '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '+', '=', '[', ']',
                '{', '}', ';', ':', ',', '.', '/', '?', '~',
            ];

            // Exclude ambiguous characters if requested
            if *exclude_ambiguous {
                let ambiguous = &['0', 'O', 'l', '1'];
                uppercase_pool.retain(|c| !ambiguous.contains(c));
                lowercase_pool.retain(|c| !ambiguous.contains(c));
                numbers_pool.retain(|c| !ambiguous.contains(c));
                symbols_pool.retain(|c| !ambiguous.contains(c));
            }

            let mut password_chars = Vec::new();
            let mut active_pools = Vec::new();

            // 1. Generate required minimums and register active pools
            if *min_uppercase > 0 {
                if uppercase_pool.is_empty() {
                    return Err("Uppercase pool is empty");
                }
                for _ in 0..*min_uppercase {
                    password_chars.push(*uppercase_pool.choose(&mut rng).ok_or("Empty pool")?);
                }
                active_pools.push(uppercase_pool.clone());
            }

            if *min_lowercase > 0 {
                if lowercase_pool.is_empty() {
                    return Err("Lowercase pool is empty");
                }
                for _ in 0..*min_lowercase {
                    password_chars.push(*lowercase_pool.choose(&mut rng).ok_or("Empty pool")?);
                }
                active_pools.push(lowercase_pool.clone());
            }

            if *min_numbers > 0 {
                if numbers_pool.is_empty() {
                    return Err("Numbers pool is empty");
                }
                for _ in 0..*min_numbers {
                    password_chars.push(*numbers_pool.choose(&mut rng).ok_or("Empty pool")?);
                }
                active_pools.push(numbers_pool.clone());
            }

            if *min_symbols > 0 {
                if symbols_pool.is_empty() {
                    return Err("Symbols pool is empty");
                }
                for _ in 0..*min_symbols {
                    password_chars.push(*symbols_pool.choose(&mut rng).ok_or("Empty pool")?);
                }
                active_pools.push(symbols_pool.clone());
            }

            // 2. If we have room left, allow all requested active pools. If none specified, error.
            if password_chars.len() < *length {
                let mut combined_pool = Vec::new();
                for pool in &active_pools {
                    combined_pool.extend(pool);
                }

                if combined_pool.is_empty() {
                    return Err(
                        "No active character pools configured (all minimum requirements are 0)",
                    );
                }

                let remaining = length - password_chars.len();
                for _ in 0..remaining {
                    password_chars.push(
                        *combined_pool
                            .choose(&mut rng)
                            .ok_or("Empty combined pool")?,
                    );
                }
            }

            // 3. Shuffle the characters so the mandated ones are mixed randomly
            password_chars.shuffle(&mut rng);

            // Calculate active pool size for entropy calculation
            let mut total_pool_size = 0;
            for pool in &active_pools {
                total_pool_size += pool.len();
            }

            if total_pool_size == 0 {
                return Err("No active character pools");
            }

            let password_string: String = password_chars.into_iter().collect();
            let entropy = (*length as f64) * (total_pool_size as f64).log2();

            Ok((password_string, entropy))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_generation() {
        let config = GeneratorConfig::Pin { length: 6 };
        let (pin, entropy) = generate(&config).unwrap();

        assert_eq!(pin.len(), 6);
        assert!(pin.chars().all(|c| c.is_ascii_digit()));
        assert!((entropy - 19.93).abs() < 0.1);
    }

    #[test]
    fn test_passphrase_custom_separators_and_capitalization() {
        // Test Hyphen + Uppercase
        let config = GeneratorConfig::Passphrase {
            words: 3,
            separator: Separator::Hyphen,
            capitalization: Capitalization::Uppercase,
        };
        let (passphrase, _) = generate(&config).unwrap();

        let split_words: Vec<&str> = passphrase.split('-').collect();
        assert_eq!(split_words.len(), 3);
        for word in split_words {
            assert!(word.chars().all(|c| c.is_uppercase() && c.is_alphabetic()));
        }

        // Test Period + TitleCase
        let config = GeneratorConfig::Passphrase {
            words: 3,
            separator: Separator::Period,
            capitalization: Capitalization::TitleCase,
        };
        let (passphrase, _) = generate(&config).unwrap();
        let split_words: Vec<&str> = passphrase.split('.').collect();
        assert_eq!(split_words.len(), 3);
        for word in split_words {
            let mut chars = word.chars();
            let first_char = chars.next().unwrap();
            assert!(first_char.is_uppercase());
            assert!(chars.all(|c| c.is_lowercase()));
        }
    }

    #[test]
    fn test_passphrase_generation_entropy() {
        let config = GeneratorConfig::Passphrase {
            words: 4,
            separator: Separator::Space,
            capitalization: Capitalization::Lowercase,
        };
        let (passphrase, entropy) = generate(&config).unwrap();

        let word_count = passphrase.split_whitespace().count();
        assert_eq!(word_count, 4);

        // 4 words * log2(7776) ≈ 51.7 bits
        assert!((entropy - 51.7).abs() < 0.1);
    }

    #[test]
    fn test_character_generation_min_counts() {
        let config = GeneratorConfig::Character {
            length: 10,
            min_uppercase: 2,
            min_lowercase: 3,
            min_numbers: 2,
            min_symbols: 3,
            exclude_ambiguous: false,
        };
        let (password, _) = generate(&config).unwrap();

        assert_eq!(password.len(), 10);

        // Count characters in each set
        let u_count = password.chars().filter(|c| c.is_uppercase()).count();
        let l_count = password
            .chars()
            .filter(|c| c.is_lowercase() && c.is_alphabetic())
            .count();
        let n_count = password.chars().filter(|c| c.is_numeric()).count();

        assert_eq!(u_count, 2);
        assert_eq!(l_count, 3);
        assert_eq!(n_count, 2);
    }

    #[test]
    fn test_character_generation_errors() {
        // Length smaller than sum of minimum requirements
        let config = GeneratorConfig::Character {
            length: 5,
            min_uppercase: 2,
            min_lowercase: 2,
            min_numbers: 2,
            min_symbols: 0,
            exclude_ambiguous: false,
        };
        assert!(generate(&config).is_err());

        // Zero length
        let config = GeneratorConfig::Character {
            length: 0,
            min_uppercase: 0,
            min_lowercase: 0,
            min_numbers: 0,
            min_symbols: 0,
            exclude_ambiguous: false,
        };
        assert!(generate(&config).is_err());
    }

    #[test]
    fn test_character_generation_exclude_ambiguous() {
        let config = GeneratorConfig::Character {
            length: 100,
            min_uppercase: 25,
            min_lowercase: 25,
            min_numbers: 25,
            min_symbols: 25,
            exclude_ambiguous: true,
        };
        let (password, _) = generate(&config).unwrap();
        let ambiguous = &['0', 'O', 'l', '1'];
        for c in password.chars() {
            assert!(
                !ambiguous.contains(&c),
                "Found ambiguous character '{}' in password: {}",
                c,
                password
            );
        }
    }

    #[test]
    fn test_character_generation_all_zero_minimums_error() {
        let config = GeneratorConfig::Character {
            length: 10,
            min_uppercase: 0,
            min_lowercase: 0,
            min_numbers: 0,
            min_symbols: 0,
            exclude_ambiguous: false,
        };
        let result = generate(&config);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "No active character pools configured (all minimum requirements are 0)"
        );
    }

    #[test]
    fn test_passphrase_zero_words_error() {
        let config = GeneratorConfig::Passphrase {
            words: 0,
            separator: Separator::Space,
            capitalization: Capitalization::Lowercase,
        };
        let result = generate(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Word count must be greater than 0");
    }

    #[test]
    fn test_pin_zero_length_error() {
        let config = GeneratorConfig::Pin { length: 0 };
        let result = generate(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Length must be greater than 0");
    }
}
