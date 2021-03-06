/*
  Copyright 2016 Robert Lathrop

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.*/

/// This module holds the game object. It has the Game struct

extern crate mio;

pub mod gameloop;
pub mod gamemap;
pub mod characters;


use glob::glob;
use std::io::prelude::*;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Vacant, Occupied};
use std::sync::Arc;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use game::gameloop::GameLoop;
use conn::server::Msg;


///This just has a hashmap of gameloops, and maps of game loops, and also holds all of the tile
///mappings
pub struct Game {
    game_loops: Mutex<HashMap<String, Arc<RefCell<GameLoop>>>>,
    pub mappings: HashMap<String, i16>,
    pub send: Sender<Msg>,
}

impl Game {
    ///Creates a new game struct. Initilizes a new hashmap, and reads the tile map file.
    pub fn new(send: Sender<Msg>) -> Game {
        Game {
            game_loops: Mutex::new(HashMap::new()),
            mappings: Game::create_mappings(),
            send: send,
        }
    }

    ///Reads the file with the paths for all images. Assigns tiles by count.
    pub fn create_mappings() -> HashMap<String, i16> {
        let mut m: HashMap<String,i16> = HashMap::new();  
        let tile_file = File::open("file_full").unwrap(); 
        let mut reader = BufReader::new(tile_file);
        let mut line: String = String::new();
        let mut count = 0;
        while reader.read_line(&mut line).unwrap() > 0 {
            m.insert(line.clone().trim().to_string(), count.clone());
            count = count + 1;
            line.clear();
        }
        for entry in glob("images/**/*.gif").unwrap() {
            match entry {
                Ok(img) => {
                    m.insert(img.file_stem().unwrap().to_str().unwrap().to_string(), count);
                    count = count + 1;
                    println!("{} {}", img.display(), count);
                },
                _ => {},
            }
        }
        m
    }

    ///Creates a new game loop with the given name, or finds it already in the hashmap.
    pub fn get_or_create_game_loop(&mut self, map_name: &str) -> Option<Arc<RefCell<GameLoop>>> {
        println!("{}", map_name);
        //This can handle all kinds of things. Checks last time user was inside, if too long it recreates. 
        //Checks the hashmap for the Gameloop. If not there, it creates a new one, adds it and returns it.
        let mut loops = self.game_loops.lock().unwrap();
        match loops.entry(map_name.to_string()) {
            Vacant(blank) => {
                match GameLoop::new(map_name, self.send.clone()) {
                    Some(game) => {
                        let full = Arc::new(RefCell::new(game));
                        blank.insert(full.clone());
                        Some(full)
                    },
                    None =>{
                        None
                    },
                }
            },
            Occupied(map) => {
                Some(map.get().clone())
            },
        }
    }
}
