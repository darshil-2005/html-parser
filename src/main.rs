mod state;

use std::fs;
use state::State;

// the character reference state uses a return state to return to the state it was invoked from.

// Most states consume one character but some might consume multiple at a time and the states are
// given as follows:
// 1) consume the character in the current state.
// 2) consume the character and change the state to another one and then reconsume the character.
// 3) switch to the next state to consume the next character.

enum DocTypeIdentifier {
    Missing,
    Available { id: String },
}

enum TokenType {
    DocType { 
        name: String,
        public_identifier: DocTypeIdentifier,
        system_identifier: DocTypeIdentifier,
        force_quirks: bool,
    },
    StartTag {
        tag_name: String,
        self_closing: bool,
        attributes: Vec<(String, String)>,
    },
    EndTag {
        tag_name: String,

        // might remove the below two in the future if there is no use for them.
        self_closing: bool,
        attributes: Vec<(String, String)>,
        //-------------------------------------
    },
    Comment {
        data: String,
    },
    Character {
        data: String,
    },
    EOF,
}

impl TokenType {
    fn new_doctype(name: String) -> Self {
        TokenType::DocType {
            name,
            public_identifier: DocTypeIdentifier::Missing,
            system_identifier: DocTypeIdentifier::Missing,
            force_quirks: false,
        }
    }

    fn new_start_tag(tag_name: String) -> Self {
        TokenType::StartTag {
            tag_name,
            self_closing: false,
            attributes: Vec::new(),
        } 
    }

    fn new_end_tag(tag_name: String) -> Self {
        TokenType::EndTag {
            tag_name,
            self_closing: false,
            attributes: Vec::new(),
        } 
    }
}

// when token is emited it must be immediately handled by tree constructor.
fn tokenizer(doc_str: &str) {
    let mut current_state: Option<State> = Some(State::Data); 
    // implement this properly
    let mut return_state: Option<State> = None;
    let mut offset: u32 = 0;
    let mut iter = doc_str.chars();

    loop {
        match current_state {

            State::Data => {
                match iter.next() {
                    Some(c) => {
                        match c {
                            '&' => {
                                return_state = Some(State::Data);
                                current_state = Some(State::CharacterReference);
                            }
                            '<' => {
                                current_state = Some(State::TagOpen);
                            }
                            '\0' => {
                                // emit an error: unexpected-null-character 
                                // emit the token.
                                println!("emit null as character token.");
                            }
                            other => {
                                //emit the other as character token
                            }
                        }
                    }
                    None => {
                        //emit an eof token
                    }
                }
            }

            State::RCData => {
                match iter.next() {
                    Some(c) => {
                        match c {
                            '&' => {
                                return_state = Some(State::RCData);
                                current_state = Some(State::CharacterReference);
                            }
                            '<' => {
                                current_state = Some(State::RCDataLessThan);
                            }
                            '\0' => {
                                // emit an error: unexpected-null-character
                                // emit a replacement error token.
                                println!("emit replacement char as character token.");
                            }
                            other => {
                                //emit the other as character token
                            }
                        }
                    }
                    None => {
                        //emit an eof token
                    }
                }
            }

            State::RawText {
                match iter.next() {
                    Some(c) => {
                        match c {
                       '<' => {
                            current_state = Some(State::RawTextLessThan);
                        }
                        '\0' => {
                            // emit an error: unexpected-null-character
                            // emit a replacement error token.
                            println!("emit replacement char as character token.");
                        }
                        other => {
                            //emit the other as character token
                        }
                        }
                    }
                    None => {
                        //emit an eof token
                    }
                }
            }

            State::ScriptData {
                match iter.next() {
                    Some(c) => {
                        match c {
                       '<' => {
                            current_state = Some(State::ScriptDataLessThan);
                        }
                        '\0' => {
                            // emit an error: unexpected-null-character
                            // emit a replacement error token.
                            println!("emit replacement char as character token.");
                        }
                        other => {
                            //emit the other as character token
                        }
                        }
                    }
                    None => {
                        //emit an eof token
                    }
                }
            }

            State::PlainText {
                match iter.next() {
                    Some(c) {
                        match c {
                        '\0' => {
                           //emit an error: unexpected-null-character. 
                        }
                        other => {
                            // emit other as a char token.
                        }
                        }
                    }
                    None {
                        // emit an eof token.
                    }
                }
            }

            State::TagOpen {
                match iter.next() {
                    Some(c) {
                        match c {
                        '!' => {
                            current_state = State::MarkupDeclarationOpen;
                        }
                        '/' => {
                            current_state = State::EndTagOpen;
                        }
                        c if c.is_ascii_alphabetic() => {
                            // Create a new start tag token, set its tag name to the empty string.
                            // Reconsume in the tag name state.
                        }
                        '?' => {
                            // emit an error: unexpected-question-mark-instead-of-tag-name
                            // Create a comment token whose data is the empty string.
                            // Reconsume in the bogus comment state.
                        } 
                        other => {
                            // emit an error: invalid-first-character-of-tag-name.
                            // Emit a U+003C LESS-THAN SIGN character token.
                            // Reconsume in the data state.
                        }
                        }
                    }
                    None {
                        //emit an error: eof-before-tag-name.
                        //emit a less than sign token and then emit an eof token.
                    }
                }
            }

            State::EndTagOpen {
                match iter.next() {
                    Some(c) {
                      match(c) {
                          c if c.is_ascii_alphabetic() => {
                            //Create a new end tag token, set its tag name to the empty string.
                            //Reconsume in the tag name state.
                          }
                          '>' => {
                              //emit an error: missing-end-tag-name
                              current_state = State::Data;
                          }
                          other => {
                              //emit an error: invalid-first-character-of-tag-name
                              //Create a comment token whose data is the empty string. 
                              //Reconsume in the bogus comment state.
                          }
                      } 
                    }

                    None {
                        //emit an error: eof-before-tag-name.
                        // emit less than char token, then solidus char token, then an eof token.
                    }

                }
            }

            State::TagName {
                match iter.next() {
                    Some(c) {
                      match(c) {
                          c if c=='\u{0009}' || c=='\u{000A}' || c=='\u{000C}' || c=='\u{0020}' => {
                              current_state = State::BeforeAttributeName;
                          }
                          '/' => {
                              current_state = State::SelfClosingStartTag;
                          }
                          '>' => {
                              current_state = State::Data;
                              // emit the current tag token
                          }
                          c if c.is_ascii_uppercase () => {
                              // emit the lowercase version of c as a char token using
                              // to_ascii_lowercase()
                              // Append the lowercase version of the current input character
                              // to the current tag token's tag name.
                          }
                          '\0' => {
                              // emit an error: unexpected-null-character
                              //Append a U+FFFD REPLACEMENT CHARACTER character to
                              //the current tag token's tag name.
                          }
                          other => {
                              // Append the current input character to the current tag token's tag name.
                          }
                      } 
                    }

                    None {
                        //emit an error: eof-in-tag.
                        // emit an eof token.
                    }
                }
            }


            State::RCDataLessThan {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '/' => {
                                // Set the temporary buffer to the empty string. 
                                // Switch to the RCDATA end tag open state.
                          }
                            other => {
                                // Emit a U+003C LESS-THAN SIGN character token. Reconsume in the RCDATA state.
                          }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }

            State::RCDataEndTagOpen {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c.is_ascii_alphabetic() {
                                // Create a new end tag token, set its tag name to the empty string.
                                // Reconsume in the RCDATA end tag name state.
                            }
                            other => {
                                // Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS character
                                // token. Reconsume in the RCDATA state.
                            }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }

            State::RCDataEndTagName {
                match iter.next() {
                    Some(c) {
                        match(c) {
                          c if c=='\u{0009}' || c=='\u{000A}' || c=='\u{000C}' || c=='\u{0020}' => {
                              // If the current end tag token is an appropriate end tag token,
                              // then switch to the before attribute name state.
                              // Otherwise, treat it as per the "anything else" entry below.
                        }
                          '/' => {
                            // If the current end tag token is an appropriate end tag token,
                            // then switch to the self-closing start tag state.
                            // Otherwise, treat it as per the "anything else" entry below.
                          }
                          '>' => {
                            // If the current end tag token is an appropriate end tag token,
                            // then switch to the data state and emit the current tag token. 
                            // Otherwise, treat it as per the "anything else" entry below.
                          }
                          c if c.is_ascii_uppercase() {
                            // Append the lowercase version of the current input character
                            // to the current tag token's tag name. 
                            // Append the current input character to the temporary buffer.
                          }
                          c if c.is_ascii_lowercase() {
                              // Append the current input character to the current tag token's tag name.
                              // Append the current input character to the temporary buffer.
                          }
                          other => {
                              // Emit a U+003C LESS-THAN SIGN character token,
                              // a U+002F SOLIDUS character token, and a character token
                              // for each of the characters in the temporary buffer
                              // (in the order they were added to the buffer).
                              // Reconsume in the RCDATA state.
                          }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }

            State::RawTextLessThan {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '/' => {
                                // Set the temporary buffer to the empty string.
                                // Switch to the RAWTEXT end tag open state.
                          }
                            other => {
                                // Emit a U+003C LESS-THAN SIGN character token.
                                // Reconsume in the RAWTEXT state.
                          }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::RawTextEndOpen{
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c.is_ascii_alphabetic() => {
                                // Create a new end tag token, set its tag name to the empty string. 
                                // Reconsume in the RAWTEXT end tag name state.
                            }
                            other => {
                                // Emit a U+003C LESS-THAN SIGN character token and a U+002F 
                                // SOLIDUS character token. Reconsume in the RAWTEXT state.
                            }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }

            State::RawTextEndTagName {
                match iter.next() {
                    Some(c) {
                        match(c) {
                          c if c=='\u{0009}' || c=='\u{000A}' || c=='\u{000C}' || c=='\u{0020}' => {
                            // If the current end tag token is an appropriate end tag token,
                            // then switch to the before attribute name state. 
                            // Otherwise, treat it as per the "anything else" entry below.
                          }
                          '/' => {
                              // spec
                          }
                          '>' => {
                              // spec
                          }
                          c if c.is_ascii_uppercase() => {
                              // spec
                          }
                          c if c.is_ascii_lowercase() => {
                              // spec
                          }
                          other => {
                              //spec
                          }

                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::ScriptDataLessThan {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '/' => {
                                //spec
                          }
                            '!' => {
                                //spec
                          }
                            other => {

                          }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }

            State::ScriptDataEndTagOpen {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c.is_ascii_alphabetic() => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::ScriptDataEndTagName {
                match iter.next() {
                    Some(c) {
                        match(c) {
                          c if c=='\u{0009}' || c=='\u{000A}' || c=='\u{000C}' || c=='\u{0020}' => {
                            //spec
                        }
                          '/' => {
                              // spec
                          }
                          '>' => {
                              // spec
                          }
                          c if c.is_ascii_uppercase() => {
                              // spec
                          }
                          c if c.is_ascii_lowercase() => {
                              // spec
                          }
                          other => {
                              //spec
                          }

                        } 
                    }

                    None {
                        //??
                    }
                }
            }


            State::ScriptDataEscapeStart {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec
                          }
                            other => {
                                //spec
                          }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }


            State::ScriptDataEscapeStartDash {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec
                          }
                            other => {
                                //spec
                          }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }


            State::ScriptDataEscaped {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec 
                          }
                            '<' => {
                                //spec
                          }
                            '\0' => {
                                //spec
                          }
                            other => {
                                //spec
                          }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }


            State::ScriptDataEscapedDash {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec 
                          }
                            '<' => {
                                //spec
                          }
                            '\0' => {
                                //spec
                          }
                            other => {
                                //spec
                          }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }

            State::ScriptDataEscapedDashDash {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec 
                          }
                            '<' => {
                                //spec
                          }
                            '\0' => {
                                //spec
                          }
                            other => {
                                //spec
                          }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }

            State::ScriptDataEscapedLessThan {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '/' => {
                                //spec
                          }
                            c if c.is_ascii_alphabetic() {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }


            State::ScriptDataEscapedEndTagOpen {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c.is_ascii_alphabetic() {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                    }
                }
            }

            State::ScriptDataEscapedEndTagName {
                match iter.next() {
                    Some(c) {
                        match(c) {
                          c if c=='\u{0009}' || c=='\u{000A}' || c=='\u{000C}' || c=='\u{0020}' => {
                            //spec
                        }
                          '/' => {
                              // spec
                          }
                          '>' => {
                              // spec
                          }
                          c if c.is_ascii_uppercase() => {
                              // spec
                          }
                          c if c.is_ascii_lowercase() => {
                              // spec
                          }
                          other => {
                              //spec
                          }

                        } 
                    }

                    None {
                        //??
                    }
               }
            }


            State::ScriptDataDoubleEscapeStart {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if (  c == '\u{0009}' ||  c=='\u{000A}' ||
                                    c=='\u{000C}' || c=='\u{0020}' || c=='\u{002F}' ||
                                    c=='\u{003E}') => {
                                //spec
                            }
                            c if c.is_ascii_uppercase() {
                                //spec
                            }
                            c if c.is_ascii_lowercase() {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }

            State::ScriptDataDoubleEscaped {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec 
                          }
                            '<' => {
                                //spec
                          }
                            '\0' => {
                                //spec
                          }
                            other => {
                                //spec
                          }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }


            State::ScriptDataDoubleEscapedDash {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec 
                          }
                            '<' => {
                                //spec
                          }
                            '\0' => {
                                //spec
                          }
                            other => {
                                //spec
                          }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }

            State::ScriptDataDoubleEscapedDashDash {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec 
                          }
                            '<' => {
                                //spec
                          }
                            '>' => {
                                //spec
                          }
                            '\0' => {
                                //spec
                          }
                            other => {
                                //spec
                          }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }

            State::ScriptDataDoubleEscapedLessThan {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '/' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::ScriptDataDoubleEscapeEnd {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if (  c == '\u{0009}' ||  c=='\u{000A}' ||
                                    c=='\u{000C}' || c=='\u{0020}' || c=='\u{002F}' ||
                                    c=='\u{003E}') => {
                                //spec
                            }
                            c if c.is_ascii_uppercase() {
                                //spec
                            }
                            c if c.is_ascii_lowercase() {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }

            State::BeforeAttributeNameState {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                            c if c == '/' || c == '>' => {
                                //spec
                            }
                            '=' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                        // spec
                    }
                }
            }


            State::AttributeName {
                //TODO: Read the docs properly for this one there is some edge case handling.
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if (c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' ||
                                c == '\u{0020}' || c == '/' || c == '>') => {
                                //spec
                            }
                            '=' => {
                                //spec
                            }
                            c if c.is_ascii_uppercase() {
                                //spec
                            }
                            '\0' => {
                                //spec
                            }

                            c if c == '"' || c == '\'' || c == '<' {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::AfterAttributeName {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                            '/' => {
                                //spec
                            }
                            '=' => {
                                //spec
                            }
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                        // spec
                    }
                }
            }

            State::BeforeAttributeName {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                            '"' => {
                                //spec
                            }
                            '\'' => {
                                //spec
                            }
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //?? 
                    }
                }
            }

            State::AttributeValueDoubleQuoted {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '"' => {
                                //spec
                            }
                            '&' => {
                                //spec
                            }
                            '\0' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }

            State::AttributeValueSingleQuoted {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '\'' => {
                                //spec
                            }
                            '&' => {
                                //spec
                            }
                            '\0' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }

            State::AttributeValueUnQuoted {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //spec
                            }
                            '&' => {
                                //spec        
                            }
                            '>' => {
                                //spec        
                            }
                            '\0' => {
                                //spec        
                            }
                            c if c == '"' || c == '\'' || c == '<' || c == '=' || c == '`' => {
                                //spec
                            }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }

            State::AfterAttributeValueQuoted {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //spec
                            }
                            '/' => {
                                //spec
                            }
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::SelfClosingStartTag {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                        //spec
                    }
                }
            }

            State::BogusComment {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '>' => {
                                //spec
                            }
                            '\0' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::MarkupDeclarationOpen {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '--' => {
                                //spec
                            }
                            //TODO: case insensitive match for 'DOCTYPE' string
                            //TODO: case insensitive match for '[CDATA[' string
                            other => {
                                //spec
                            }
                        } 
                    }

                    None {
                        //??
                    }
                }
            }

            State::CommentStart {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec
                            }
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::CommentStartDash {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec
                            }
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::Comment {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '<' => {
                                //spec
                            }
                            '-' => {
                                //spec
                            }
                            '\0' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::CommentLessThan {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '!' => {
                                //spec
                            }
                            '<' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::CommentLessThanBang {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::CommentLessThanBangDash {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::CommentLessThanBangDashDash {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::CommentEndDash {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::CommentEnd {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '>' => {
                                //spec
                            }
                            '!' => {
                                //spec
                            }
                            '-' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::CommentEndBang {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '-' => {
                                //spec
                            }
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::Doctype {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //spec
                            }
                           '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::BeforeDoctypeName {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                           c if c.is_ascii_uppercase() => {
                                //spec
                            }
                           '\0' => {
                               //spec
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::DoctypeName {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //spec
                            }
                           '>' => {
                               //spec
                            }
                           c if c.is_ascii_uppercase() => {
                                //spec
                            }
                           '\0' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::AfterDoctypeName {
                //TODO: read the doc properly while implementing this.
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::AfterDoctypePublicKeyword {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //spec
                            }
                           '"' => {
                               //spec
                            }
                           '\'' => {
                               //spec
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::BeforeDoctypePublicIdentifier {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                           '"' => {
                               //spec
                            }
                           '\'' => {
                               //spec
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::DoctypePublicIdentifierDoubleQuoted {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '"' => {
                                //spec
                            }
                            '\0' => {
                                //spec
                            }
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::DoctypePublicIdentifierSingleQuoted {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            '\'' => {
                                //spec
                            }
                            '\0' => {
                                //spec
                            }
                            '>' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::AfterDoctypePublicIdentifier {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //spec
                            }
                           '>' => {
                               //spec
                            }
                           '"' => {
                               //spec
                            }
                           '\'' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::BetweenDoctypePublicAndSystemIdentifiers {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                           '>' => {
                               //spec
                            }
                           '"' => {
                               //spec
                            }
                           '\'' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::AfterDoctypeSystemKeyword {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                          '"' => {
                               //spec
                            }
                           '\'' => {
                               //spec
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::BeforeDoctypeSystemIdentifier {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                          '"' => {
                               //spec
                            }
                           '\'' => {
                               //spec
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::DoctypeSystemIdentifierDoubleQuoted {
                match iter.next() {
                    Some(c) {
                        match(c) {
                          '"' => {
                               //spec
                            }
                           '\0' => {
                               //spec
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::DoctypeSystemIdentifierSingleQuoted {
                match iter.next() {
                    Some(c) {
                        match(c) {
                          '\'' => {
                               //spec
                            }
                           '\0' => {
                               //spec
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::AfterDoctypeSystemIdentifier {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0009}' || c == '\u{000A}' || c == '\u{000C}' || c == '\u{0020}' => {
                                //ignore
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::BogusDoctype {
                match iter.next() {
                    Some(c) {
                        match(c) {
                           '>' => {
                               //spec
                            }
                           '\0' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::CDataSection {
                match iter.next() {
                    Some(c) {
                        match(c) {
                           ']' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //spec
                    }
                }
            }

            State::CDataSectionBracket {
                match iter.next() {
                    Some(c) {
                        match(c) {
                           ']' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::CDataSectionEnd {
                match iter.next() {
                    Some(c) {
                        match(c) {
                           ']' => {
                               //spec
                            }
                           '>' => {
                               //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::CharacterReference {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c.is_ascii_alphanumeric() => {
                                //spec
                            }
                            '#' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::NamedCharacterReference {
                //TODO: Complicated to implement use proper documentation to implement this.
                match iter.next() {
                    Some(c) {
                        match(c) {
                        } 
                    }
                    None {
                    }
                }
            }

            State::AmbiguousAmpersand {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c.is_ascii_alphanumeric() {
                                //spec
                            }
                            ';' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::NumericCharacterReference {
                match iter.next() {
                    Some(c) {
                        match(c) {
                            c if c == '\u{0078}' || c == '\u{0058}' => {
                                //spec
                            }
                            other => {
                                //spec
                            }
                        } 
                    }
                    None {
                        //??
                    }
                }
            }

            State::HexadecimalCharacterReferenceStart {
                //TODO: Complicated to implement use the docs carefully.
                match iter.next() {
                    Some(c) {
                        match(c) {
                        } 
                    }
                    None {
                    }
                }
            }

            State::DecimalCharacterReferenceStart {
                //TODO: Complicated to implement use the docs carefully.
                match iter.next() {
                    Some(c) {
                        match(c) {
                        } 
                    }

                    None {
                    }
                }
            }

            State::HexadecimalCharacterReference {
                //TODO: Complicated to implement use the docs carefully.
                match iter.next() {
                    Some(c) {
                        match(c) {
                        } 
                    }

                    None {
                    }
                }
            }

            State::DecimalCharacterReference {
                //TODO: Complicated to implement use the docs carefully.
                match iter.next() {
                    Some(c) {
                        match(c) {
                        } 
                    }

                    None {
                    }
                }
            }

            State::NumericCharacterReferenceEnd {
                //TODO: Complicated to implement use the docs carefully.
                match iter.next() {
                    Some(c) {
                        match(c) {
                        } 
                    }

                    None {
                    }
                }
            }



        }
        println!("{}", c);
    }
}

fn read_file(path: &str) -> Vec<u8> {
    let bytes = fs::read(path).expect("failed to read file");
    bytes
}

fn main() {
    let bytes: Vec<u8> = read_file("index.html");

    //Add proper decoding
    let text = std::str::from_utf8(&bytes).expect("file is not valid utf8");
    tokenizer(text);
}




