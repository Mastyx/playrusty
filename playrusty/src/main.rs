#![allow(unused_imports, unused_variables, dead_code)]
// silenzia i warning

use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::thread;
use std::time::Duration;

// struttura e implementazione AudioaPlayer
struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
}
impl AudioPlayer {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(AudioPlayer { _stream, sink })
    }

    fn load_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // preleva il file dal filesystem
        let file = File::open(path)?;
        //crea un buffer reader per leggere il file
        let reader = BufReader::new(file);
        // decoder di rodio che riconosce automaticamente il formato
        let source = Decoder::new(reader)?;

        // aggiunge il file decodificato alla coda del sink
        self.sink.append(source);
        println!("File audio caricato : {}", path);
        Ok(())
    }

    // avvia o riprende la riproduzione del brano
    fn play(&self) {
        self.sink.play();
        println!(" ▶️ Riproduzione Avviata ");
    }

    fn pause(&self) {
        self.sink.pause();
        println!("Riproduzione in pausa");
    }

    fn stop(&self) {
        self.sink.stop();
        println!("Riproduzione fermata");
    }
    // imposta il volume
    fn set_volume(&self, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        self.sink.set_volume(vol);
        println!("Volume impostato a : {:.0}%", vol * 100.0);
    }
    // controlla se il sink e vuoto (nessun file in coda)
    fn is_empty(&self) -> bool {
        self.sink.empty()
    }
}

// funzione che stampa un help
// dei comandi disponibili
fn print_help() {
    println!("\n######## Comandi disponibili ########\n ");
    println!("load <file>       - carica un file ");
    println!("play              - avvia la riproduzione ");
    println!("pause             - mette in pausa ");
    println!("stop              - ferma la riproduzione ");
    println!("voloume <0-1>     - imposta il volume es. 0.5 ");
    println!("help              - mostra questo aiuto ");
    println!("quit              - esci ");
    println!("----------------------------------------------");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Player Audio in Rust");
    println!("Formati supportati mp3, wav, flac, ogg");
    print_help();

    let player = AudioPlayer::new()?;
    let mut input = String::new();

    // loop principale del programma
    loop {
        //stampa il prompt
        print!("> ");
        io::stdout().flush()?;
        input.clear();

        // legge una riga dell'input da tastiera
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // rimozione degli spazi vuoti
                let command = input.trim();
                // divide il comando in parti separate da spazi
                let parts: Vec<&str> = command.split_whitespace().collect();

                // se non ce nessun comando continua il loop
                if parts.is_empty() {
                    continue;
                }

                // pattern matching sul primo elemento (il comando)
                match parts[0] {
                    //
                    "load" => {
                        // controlal se e stato fornito un nome file
                        if parts.len() < 2 {
                            println!(" X Uso: load <percorso file> ");
                            continue;
                        }
                        // ricostruisce il percorso del file (gestisce gli spazi)
                        let file_path = parts[1..].join(" ");

                        // tenta di caricare il file
                        match player.load_file(&file_path) {
                            Ok(_) => {}
                            Err(e) => println!("X Errore nel caricamento : {} ", e),
                        }
                    }
                    "play" => {
                        // controlla se ce un file caricato prima di riprodurre
                        if player.is_empty() {
                            println!("Attenzione!!! Nessun file. Usa 'load <file>' prima ");
                        } else {
                            player.play();
                        }
                    }
                    "pause" => {
                        player.pause();
                    }
                    "stop" => {
                        player.stop();
                    }
                    "volume" => {
                        // controlla se e stato  fornito un valore per il volume
                        if parts.len() < 2 {
                            println!("Uso volume da 0.0 a 1.0");
                            continue;
                        }
                        // tente di parsare il volume come numero decimale
                        match parts[1].parse::<f32>() {
                            Ok(vol) => player.set_volume(vol),
                            Err(_) => println!("X Volume deve essere un numero tra 0.0 e 1.0"),
                        }
                    }
                    "help" => {
                        print_help();
                    }
                    "quit" | "exit" | "q" => {
                        println!("Ciao");
                        break; // esce dal loop
                    }
                    //comando non riconosciuto
                    _ => {
                        println!("Comando non riconsociuto");
                    }
                }
            }

            // gestisce errore della lettura dell input
            Err(e) => {
                println!("Errore nella lettura dell'input : {}", e);
                break;
            }
        }
        // piccola pausa per evitare che il loop consumi troppa cpu
        thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
