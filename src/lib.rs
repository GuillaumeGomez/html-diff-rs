// Copyright 2014 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate tendril;
extern crate html5ever;

use std::io;
use std::default::Default;
use std::rc::Rc;

use tendril::{ByteTendril, ReadExt};

use html5ever::tokenizer::{TokenSink, Tokenizer, Token, TokenizerOpts, ParseError, TokenSinkResult};
use html5ever::tokenizer::{CharacterTokens, NullCharacterToken, TagToken, StartTag, EndTag};
use html5ever::tokenizer::buffer_queue::BufferQueue;

#[derive(Debug)]
struct TreeToken {
    token: Token,
    children: Vec<TreeToken>,
    parent: Option<Box<TreeToken>>,
}

impl TreeToken {
    fn new(token: Token, parent: Option<Box<TreeToken>>) -> TreeToken {
        TreeToken {
            token: token,
            children: Vec::new(),
            parent: parent,
        }
    }
}

#[derive(Debug)]
struct TokenPrinter {
    in_char_run: bool,
    tokens: Vec<TreeToken>,
    current: Option<Box<TreeToken>>,
}

impl TokenPrinter {
    fn is_char(&mut self, is_char: bool) {
        match (self.in_char_run, is_char) {
            (false, true ) => print!("CHAR : \""),
            (true,  false) => println!("\""),
            _ => (),
        }
        self.in_char_run = is_char;
    }

    fn do_char(&mut self, c: char) {
        self.is_char(true);
        print!("{}", c.escape_default().collect::<String>());
    }
}

impl TokenSink for TokenPrinter {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        match token {
            CharacterTokens(b) => {
                for c in b.chars() {
                    self.do_char(c);
                }
            }
            NullCharacterToken => self.do_char('\0'),
            TagToken(ref tag) => {
                self.is_char(false);
                // This is not proper HTML serialization, of course.
                match tag.kind {
                    StartTag => {
                        //print!("TAG  : <\x1b[32m{}\x1b[0m", tag.name);
                        let res = { self.current.is_some() };
                        if res {
                            let tmp = Box::new(TreeToken::new(token, self.current));
                            self.current = Some(tmp);
                        }
                    }
                    EndTag => {
                        //print!("TAG  : <\x1b[31m/{}\x1b[0m", tag.name),
                        if self.current.is_none() {
                            println!("Huge issue in here!");
                            return TokenSinkResult::Continue;
                        }
                        let mut parent = self.current.unwrap().parent;
                        if let Some(mut parent) = parent {
                            parent.children.push(*self.current.unwrap());
                        } else {
                            self.tokens.push(*self.current.unwrap());
                        }
                        self.current = parent;
                    }
                }
                for attr in tag.attrs.iter() {
                    print!(" \x1b[36m{}\x1b[0m='\x1b[34m{}\x1b[0m'",
                        attr.name.local, attr.value);
                }
                if tag.self_closing {
                    print!(" \x1b[31m/\x1b[0m");
                }
                println!(">");
            }
            ParseError(err) => {
                self.is_char(false);
                println!("ERROR: {}", err);
            }
            _ => {
                self.is_char(false);
                println!("OTHER: {:?}", token);
            }
        }
        TokenSinkResult::Continue
    }
}

pub fn entry_point<T: io::Read, U: io::Read>(content1: &mut T, content2: &mut U) {
    let mut sink = TokenPrinter {
        in_char_run: false,
        tokens: Vec::new(),
        current: None,
    };
    let mut chunk = ByteTendril::new();
    content1.read_to_tendril(&mut chunk).unwrap();
    let mut input = BufferQueue::new();
    input.push_back(chunk.try_reinterpret().unwrap());

    let mut tok = Tokenizer::new(sink, TokenizerOpts::default());
    let _ = tok.feed(&mut input);
    assert!(input.is_empty());
    tok.end();
    sink.is_char(false);
    println!("{:?}", sink);
}
