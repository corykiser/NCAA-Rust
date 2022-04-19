mod game;

extern crate csv;
use rand::prelude::*;

use csv::StringRecord;
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

#[derive(Debug)]
struct Game {
    team1: u32,
    team2: u32,
    team1rating: f64,
    team2rating: f64,
    team1prob: f64,
    team2prob: f64,
    winnerprob: f64,
    winner: u32,
}
struct Rounds {
    round1: [[i32; 2]; 8],
    round2: [[i32; 4]; 4],
    round3: [[i32; 8]; 2],
    round4: [[i32; 16]; 1],
}

struct Bracket {
    round1: Vec<Game>, //round of 64
    round2: Vec<Game>, //round of 32
    round3: Vec<Game>, //round of 16
    round4: Vec<Game>, //round of 8
    round5: Vec<Game>, //round of 4
    round6: Vec<Game>, //round of 2
    winner: u32,
    prob: f64,
    score: f64,
    adj_score: f64,
}
//todo impl Bracket score, pass in reference to seed_lookup

fn main() {

    //structure of tourney
    let round1: [[i32; 2]; 8] = [
        [1, 16],
        [2, 15],
        [3, 14],
        [4, 13],
        [5, 12],
        [6, 11],
        [7, 10],
        [8, 9],
    ];
    let round2: [[i32; 4]; 4] = [
        [1, 16, 8, 9],
        [5, 12, 4, 13],
        [6, 11, 3, 14],
        [7, 10, 2, 15],
    ];
    let round3: [[i32; 8]; 2] = [[1, 16, 8, 9, 5, 12, 4, 13], [6, 11, 3, 14, 7, 10, 2, 15]];
    let round4: [[i32; 16]; 1] = [[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]];

    //put arrays into struct to make it easy to feed multiple arrays into Bracket functions
    let rounds = Rounds{round1: round1, round2: round2, round3: round3, round4: round4};

    //use pop method to remove last char of string, it returns it in an Option Some(T)
    // remove_matches method will take out strings
    //let x: &[_] = &['1', '2'];
    // assert_eq!("12foo1bar12".trim_matches(x), "foo1bar");
    //assert_eq!("1foo1barXX".trim_matches(|c| c == '1' || c == 'X'), "foo1bar");
    //assert!(zero.is_ascii_digit());

    let file_path = "/Users/corydkiser/Documents/ncaa/fivethirtyeight_ncaa_forecasts.csv";
    //.expect below goes from Ok(T) to T
    let mut rdr = csv::Reader::from_path(file_path).expect("file access error");

    let mut mensrecords: Vec<StringRecord> = Vec::new();
    // Loop over each record.
    for result in rdr.records() {
        let record = &result.unwrap();
        if record[0].starts_with("mens")
            && record[1].contains("2022-03-16")
            && record[3].contains("1.0")
        {
            mensrecords.push(record.clone());
        }
    }
    //create containers
    let mut rating: [f64; 64] = [0.0; 64];
    let mut teamid: [u32; 64] = [0; 64];
    let mut seed: [u32; 64] = [0; 64];
    let mut name: Vec<String> = Vec::new();
    let mut region: Vec<String> = Vec::new();
    //
    for i in 0..mensrecords.len() {
        rating[i] = mensrecords[i][14].parse().unwrap(); //populate Team Ratings array
        teamid[i] = mensrecords[i][12].parse().unwrap(); //populate team id array
        name.push(mensrecords[i][13].to_string()); //populate team name array
        region.push(mensrecords[i][15].to_string()); //populate regions array
                                                     //below removes non ascii digits "a" and "b" from team seeds
                                                     //let test = "12b3as>";
                                                     //let test2: String = test.to_string().chars().filter(|x| x.is_ascii_digit()).collect();
        if mensrecords[i][16].ends_with("a") || mensrecords[i][16].ends_with("b") {
            //let length = mensrecords[i][16].len();
            let mut tempstring = mensrecords[i][16].to_string();
            tempstring.pop(); //this returns the removed value
            seed[i] = tempstring.parse().unwrap(); //populate team seed array
        } else{
            seed[i] = mensrecords[i][16].parse().unwrap();
        }
    }

    //creat HashMaps (dictionaries) to look up values fast
    let mut name_lookup = HashMap::new();
    let mut region_lookup: HashMap<u32, &String> = HashMap::new();
    let mut seed_lookup: HashMap<u32, u32> = HashMap::new();
    let mut id_lookup = HashMap::new();
    let mut rating_lookup = HashMap::new();
    //populate HashMaps created above
    for i in 0..mensrecords.len() {
        name_lookup.insert(teamid[i], &name[i]);
        region_lookup.insert(teamid[i], &region[i]);
        seed_lookup.insert(teamid[i], seed[i]);
        //println!("{}",seed[i]);
        rating_lookup.insert(teamid[i], rating[i]);
        id_lookup.insert(&name[i], teamid[i]);
    }
    // name_lookup.get(teamid[2]).unwrap();  example lookup

    //println!("{}", seed_lookup.get(&2250).unwrap());

    let mut east: Vec<u32>= Vec::with_capacity(16);
    let mut midwest: Vec<u32>= Vec::with_capacity(16);
    let mut south: Vec<u32>= Vec::with_capacity(16);
    let mut west: Vec<u32>= Vec::with_capacity(16);

    //for some reason match didnt work
    //sloppy, but had to use for-if
    for i in 0..64 {
        if region.get(i).unwrap().to_owned() == "East".to_string(){
            east.push(teamid[i]);
        }
        if region.get(i).unwrap().to_owned() == "West".to_string(){
            west.push(teamid[i]);
        }
        if region.get(i).unwrap().to_owned() == "Midwest".to_string(){
            midwest.push(teamid[i]);
        }
        if region.get(i).unwrap().to_owned() == "South".to_string(){
            south.push(teamid[i]);
        }
    }
    // combine regions into one vector to make it easy to pass into functions
    let regions: Vec<Vec<u32>> = vec![east, west, midwest, south];


    for _ in 0..1000000 {
        //let game1 = new_game(2305, 2250, &rating_lookup);
        let b1 = new_bracket(&rounds, &regions, &seed_lookup, &rating_lookup);
        println!("{:?}", name_lookup.get(&b1.winner).unwrap());
        println!("{:?}", b1.adj_score);
        println!();
    }

}

fn print_bracket(bracket: &Bracket, name_lookup: HashMap<u32, &String>){
    println!("{:?}", name_lookup.get(&bracket.winner).unwrap());
    println!("{:?}", bracket.adj_score);
    println!();
}

fn win_prob(rating1: &f64, rating2: &f64) -> f64 {
    let diff = rating1 - rating2;
    let prob = 1.0 / (1.0 + 10.0_f64.powf(-1.0 * diff * 30.464 / 400.0));
    prob
}

fn new_game(
    team1: u32,
    team2: u32,
    rating_lookup: &HashMap<u32, f64>,
) -> Game {
    let team1rating = rating_lookup.get(&team1).unwrap();
    let team2rating = rating_lookup.get(&team2).unwrap();
    let team1winprob = win_prob(team1rating, team2rating);
    let team2winprob = 1.0 - team1winprob;

    let mut rng = rand::thread_rng();
    let y: f64 = rng.gen(); // generates a float between 0 and 1

    let mut winner = team1;
    let mut winnerprobtemp = team1winprob;

    if team1winprob < y {
        winner = team2;
        winnerprobtemp = team2winprob;
    }
    Game {
        team1: team1,
        team2: team2,
        team1rating: *team1rating,
        team2rating: *team2rating,
        team1prob: team1winprob,
        team2prob: team2winprob,
        winnerprob: winnerprobtemp,
        winner: winner,
    }
}

//todo new_bracket function pass in reference to roundinfo and regions
fn new_bracket(rounds: &Rounds, regions: &Vec<Vec<u32>>, seed_lookup: &HashMap<u32, u32>, rating_lookup: &HashMap<u32, f64>) -> Bracket {
    //todo change () to Bracket
    let mut games1: Vec<Game> = Vec::with_capacity(32);
    let mut games2: Vec<Game> = Vec::with_capacity(16);
    let mut games3: Vec<Game> = Vec::with_capacity(8);
    let mut games4: Vec<Game> = Vec::with_capacity(4);
    let mut games5: Vec<Game> = Vec::with_capacity(2);
    let mut games6: Vec<Game> = Vec::with_capacity(1);

    let mut checklist: HashSet<u32> = HashSet::with_capacity(64);

    let mut games1winners: Vec<u32> = Vec::with_capacity(32);
    let mut games2winners: Vec<u32> = Vec::with_capacity(16);
    let mut games3winners: Vec<u32> = Vec::with_capacity(8);
    let mut games4winners: Vec<u32> = Vec::with_capacity(4);
    let mut games5winners: Vec<u32> = Vec::with_capacity(2);
    let mut games6winners: Vec<u32> = Vec::with_capacity(1);

    for region in regions{ //for each region
        for game in rounds.round1{
            for team1 in region{
                let x = *seed_lookup.get(team1).unwrap();
                if game[0] as u32 == *seed_lookup.get(team1).unwrap(){
                    for team2 in region{
                        if game[1] as u32 == *seed_lookup.get(team2).unwrap() && !checklist.contains(team1) && !checklist.contains(team2){
                            let gamesim = new_game(*team1,*team2,&rating_lookup);
                            games1winners.push(gamesim.winner);
                            games1.push(gamesim);
                            checklist.insert(*team1);
                            checklist.insert(*team2);
                        }
                    }
                }
            }
        }
    }
    //confirm 32 games are in games1 vec
    assert!(games1winners.len() == 32);
    assert!(checklist.len() == 64);
    checklist.clear();

    for region in regions{
        for game in rounds.round2{
            //look in the winners list for teams in correct region and that meet seed criteria
            let matchup: Vec<&u32> = games1winners.iter().filter(|x| region.contains(x) && game.contains(&(*seed_lookup.get(x).unwrap() as i32))).collect();
            let gamesim = new_game(*matchup[0],*matchup[1] ,&rating_lookup);
            games2winners.push(gamesim.winner);
            games2.push(gamesim);
        }
    }
    assert!(games2winners.len() == 16);

    for region in regions{
        for game in rounds.round3{
            //look in the winners list for teams in correct region and that meet seed criteria
            let matchup: Vec<&u32> = games2winners.iter().filter(|x| region.contains(x) && game.contains(&(*seed_lookup.get(x).unwrap() as i32))).collect();
            let gamesim = new_game(*matchup[0],*matchup[1] ,&rating_lookup);
            games3winners.push(gamesim.winner);
            games3.push(gamesim);
        }
    }
    assert!(games3winners.len() == 8);

    for region in regions{
        for game in rounds.round4{
            //look in the winners list for teams in correct region and that meet seed criteria
            let matchup: Vec<&u32> = games3winners.iter().filter(|x| region.contains(x) && game.contains(&(*seed_lookup.get(x).unwrap() as i32))).collect();
            let gamesim = new_game(*matchup[0],*matchup[1] ,&rating_lookup);
            games4winners.push(gamesim.winner);
            games4.push(gamesim);
        }
    }
    assert!(games4winners.len() == 4);

    //Match South with Midwest
    let matchup: Vec<&u32> = games4winners.iter().filter(|x| regions[0].contains(x) || regions[1].contains(x)).collect();
    let gamesim = new_game(*matchup[0],*matchup[1] ,&rating_lookup);
    games5winners.push(gamesim.winner);
    games5.push(gamesim);
    // match east and west
    let matchup: Vec<&u32> = games4winners.iter().filter(|x| regions[2].contains(x) || regions[3].contains(x)).collect();
    let gamesim = new_game(*matchup[0],*matchup[1] ,&rating_lookup);
    games5winners.push(gamesim.winner);
    games5.push(gamesim);

    assert!(games5winners.len() == 2);

    //FINAL GAME!!!
    let gamesim = new_game(games5winners[0],games5winners[1] ,&rating_lookup);
    let winner = gamesim.winner;
    games6winners.push(gamesim.winner);
    games6.push(gamesim);

    assert!(games6winners.len() == 1);
    //SIMULATIONS OVER

    //Probability Calculations
    let round1prob: f64 = games1.iter().map(|x|x.winnerprob).product();
    let round2prob: f64 = games2.iter().map(|x|x.winnerprob).product();
    let round3prob: f64 = games3.iter().map(|x|x.winnerprob).product();
    let round4prob: f64 = games4.iter().map(|x|x.winnerprob).product();
    let round5prob: f64 = games5.iter().map(|x|x.winnerprob).product();
    let round6prob: f64 = games6.iter().map(|x|x.winnerprob).product();
    let prob = round1prob * round2prob * round3prob * round4prob * round5prob * round6prob;


    let round1score: u32 = games1.iter().map(|x|seed_lookup.get(&x.winner).unwrap() + 1).sum();
    let round2score: u32 = games2.iter().map(|x|seed_lookup.get(&x.winner).unwrap() + 2).sum();
    let round3score: u32 = games3.iter().map(|x|seed_lookup.get(&x.winner).unwrap() + 4).sum();
    let round4score: u32 = games4.iter().map(|x|seed_lookup.get(&x.winner).unwrap() * 8).sum();
    let round5score: u32 = games5.iter().map(|x|seed_lookup.get(&x.winner).unwrap() * 16).sum();
    let round6score: u32 = games6.iter().map(|x|seed_lookup.get(&x.winner).unwrap() * 32).sum();
    let score: f64 = (round1score + round2score + round3score + round4score + round5score + round6score) as f64;


    Bracket {
        round1: games1, //round of 64
        round2: games2, //round of 32
        round3: games3, //round of 16
        round4: games4, //round of 8
        round5: games5, //round of 4
        round6: games6, //round of 2
        winner: winner,
        prob: prob,
        score: score,
        adj_score: prob * score,
    }



}


//todo score_bracket function pass in reference to seed_lookup
//todo print_bracket pass in reference to name_lookup