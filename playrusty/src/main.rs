#![allow(unused_imports, unused_variables, dead_code)]
// silenzia i warning

use rodio::{Decoder, OutputStream, Sink};
use std::fs::{read_dir, File};
use std::io::{self, BufReader, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

// struttura e implementazione AudioPlayer
struct AudioPlayer {
    _stream: OutputStream,                    // stream input audio
    stream_handle: rodio::OutputStreamHandle, // handle che controlla lo stream
    sink: Sink,                               // sink di rodio per gestiore la riproduzione
    playlist: Vec<String>,                    // lista dei file nella playlist
    current_track: usize,                     // indice del brano corrente
    playlist_active: bool,                    // indica se la playlist è attiva
    playlist_len: usize,                      // lunghezza della playlist per tracciamento
}

impl AudioPlayer {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        //crea lo stream di output di default
        let (_stream, stream_handle) = OutputStream::try_default()?;

        // crea un nuovo sink
        let sink = Sink::try_new(&stream_handle)?;
        Ok(AudioPlayer {
            _stream,
            stream_handle,
            sink,
            playlist: Vec::new(),
            current_track: 0,
            playlist_active: false,
            playlist_len: 0,
        })
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
        println!("▶️  Riproduzione Avviata ");
    }

    fn pause(&self) {
        self.sink.pause();
        println!("⸸️  Riproduzione in pausa");
    }

    fn stop(&mut self) {
        self.sink.stop();
        self.playlist_active = false;
        println!("⹸️  Riproduzione fermata");
    }

    // imposta il volume
    fn set_volume(&self, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        // clamp assicura che il volume sia nei limiti
        self.sink.set_volume(vol);
        println!("🔊 Volume impostato a : {:.0}%", vol * 100.0);
    }

    // controlla se il sink e vuoto (nessun file in coda)
    fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    // carica la playlist da una cartella
    fn load_playlist(&mut self, folder_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // verifica se la cartella esiste
        let path = Path::new(folder_path);
        if !path.exists() {
            return Err(format!("Cartella '{}' non trovata ", folder_path).into());
        }

        // svuota la playlist precedente
        self.playlist.clear();
        self.current_track = 0;
        self.playlist_active = false;

        // estensioni audio supportate
        let audio_extensions = ["mp3", "wav", "flac", "m4a", "ogg"];

        // legge tutti i file nella Cartella
        for entry in read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            // controlla se e un file e non una cartella
            if file_path.is_file() {
                // estrae l'estensione
                if let Some(extension) = file_path.extension() {
                    // Correzione: convertire OsStr a str
                    if let Some(ext_str) = extension.to_str() {
                        // se l'estensione e supportata aggiunge alla playlist
                        if audio_extensions.contains(&ext_str.to_lowercase().as_str()) {
                            if let Some(path_str) = file_path.to_str() {
                                self.playlist.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        // ordina la playlist alfabeticamente
        self.playlist.sort();

        println!("🎵 Playlist caricata con {} brani", self.playlist.len());
        for (i, track) in self.playlist.iter().enumerate() {
            // Estrae solo il nome del file senza il percorso
            let filename = Path::new(track)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("File sconosciuto");
            println!("  {}. {}", i + 1, filename);
        }

        Ok(())
    }

    // Avvia la riproduzione della playlist
    fn play_playlist(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.playlist.is_empty() {
            println!("⚠ Nessuna playlist caricata. Usa 'playlist run <cartella>' prima.");
            return Ok(());
        }

        // Ferma qualsiasi riproduzione corrente
        self.sink.stop();

        // Reset del brano corrente
        self.current_track = 0;

        // Carica tutti i file della playlist nel sink
        for track_path in self.playlist.iter() {
            match self.load_file_to_sink(track_path) {
                Ok(_) => {
                    let filename = Path::new(track_path)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("File sconosciuto");
                    println!("  ✅ Aggiunto: {}", filename);
                }
                Err(e) => {
                    println!("  ❌ Errore caricando {}: {}", track_path, e);
                }
            }
        }

        // Avvia la riproduzione
        self.sink.play();
        self.playlist_active = true;
        self.playlist_len = self.playlist.len();

        println!("🎵 Playlist avviata! {} brani in coda", self.playlist.len());

        // Mostra il primo brano in riproduzione
        self.show_current_track();

        Ok(())
    }

    // Funzione helper per caricare file nel sink senza stampare messaggi
    fn load_file_to_sink(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader)?;
        self.sink.append(source);
        Ok(())
    }

    // Mostra il brano attualmente in riproduzione
    fn show_current_track(&self) {
        if !self.playlist.is_empty() && self.current_track < self.playlist.len() {
            let current_path = &self.playlist[self.current_track];
            let filename = Path::new(current_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("File sconosciuto");

            println!(
                "\n🎵 Ora in riproduzione: {} ({}/{}) 🎵",
                filename,
                self.current_track + 1,
                self.playlist.len()
            );
        }
    }

    // Monitora lo stato della playlist e aggiorna il brano corrente
    fn check_playlist_progress(&mut self) {
        if !self.playlist_active || self.playlist.is_empty() {
            return;
        }

        // Calcola approssimativamente quale brano dovrebbe essere in riproduzione
        // basandosi sul numero di brani rimanenti nel sink
        let remaining_in_queue = self.sink.len();
        let expected_current = self.playlist_len.saturating_sub(remaining_in_queue);

        // Aggiorna solo se c'è stato un cambio
        if expected_current > self.current_track && expected_current <= self.playlist_len {
            self.current_track = expected_current;

            if self.current_track < self.playlist.len() {
                self.show_current_track();
            }
        }

        // Se il sink è vuoto, la playlist è finita
        if self.sink.empty() && self.playlist_active {
            println!("\n🎵 Playlist completata! 🎵\n");
            self.playlist_active = false;
        }
    }

    // Mostra informazioni sulla playlist corrente
    fn show_playlist(&self) {
        if self.playlist.is_empty() {
            println!("📋 Nessuna playlist caricata");
            return;
        }

        println!("\n📋 Playlist corrente ({} brani):", self.playlist.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for (i, track) in self.playlist.iter().enumerate() {
            let filename = Path::new(track)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("File sconosciuto");

            // Indica il brano corrente (se stiamo riproducendo)
            let (indicator, status) = if i == self.current_track && self.playlist_active {
                ("▶️", " ← ORA IN RIPRODUZIONE")
            } else if i < self.current_track && self.playlist_active {
                ("✅", " (completato)")
            } else {
                ("⏸️", "")
            };

            println!("{} {:2}. {}{}", indicator, i + 1, filename, status);
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}

// funzione che stampa un help
// dei comandi disponibili
fn print_help() {
    println!("\n 📻 🎵Comandi disponibili🎵 📻\n ");
    println!("load <file>       - carica un file ");
    println!("play              - avvia la riproduzione ");
    println!("pause             - mette in pausa ");
    println!("stop              - ferma la riproduzione ");
    println!("volume <0-1>      - imposta il volume es. 0.5 ");
    println!("playlist run <cartella> - carica e avvia playlist ");
    println!("playlist show     - mostra playlist corrente ");
    println!("playlist status   - aggiorna e mostra stato playlist ");
    println!("help              - mostra questo aiuto ");
    println!("quit              - esci ");
    println!("----------------------------------------------");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Player Audio in Rust");
    println!("Formati supportati mp3, wav, flac, ogg");
    print_help();

    let mut player = AudioPlayer::new()?;
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
                    "load" => {
                        // controlla se e stato fornito un nome file
                        if parts.len() < 2 {
                            println!("❌ Uso: load <percorso file> ");
                            continue;
                        }
                        // ricostruisce il percorso del file (gestisce gli spazi)
                        let file_path = parts[1..].join(" ");

                        // tenta di caricare il file
                        match player.load_file(&file_path) {
                            Ok(_) => {}
                            Err(e) => println!("❌ Errore nel caricamento : {} ", e),
                        }
                    }
                    "play" => {
                        // controlla se ce un file caricato prima di riprodurre
                        if player.is_empty() {
                            println!("⚠️ Attenzione!!! Nessun file. Usa 'load <file>' prima ");
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
                        // controlla se e stato fornito un valore per il volume
                        if parts.len() < 2 {
                            println!("Uso volume da 0.0 a 1.0");
                            continue;
                        }
                        // tenta di parsare il volume come numero decimale
                        match parts[1].parse::<f32>() {
                            Ok(vol) => player.set_volume(vol),
                            Err(_) => println!("❌ Volume deve essere un numero tra 0.0 e 1.0"),
                        }
                    }
                    "help" => {
                        print_help();
                    }
                    "quit" | "exit" | "q" => {
                        println!("Ciao! 👋");
                        break; // esce dal loop
                    }
                    // cartella playlist
                    "playlist" => {
                        // gestisce i sottocomandi playlist
                        if parts.len() < 2 {
                            println!(
                                "Uso: playlist run <cartella> | playlist show | playlist status"
                            );
                            continue;
                        }
                        match parts[1] {
                            "run" => {
                                if parts.len() < 3 {
                                    println!("Uso: playlist run <cartella>");
                                    continue;
                                }
                                // ricostruisce il percorso della cartella
                                let folder_path = parts[2..].join(" ");

                                //carica e avvia la playlist
                                match player.load_playlist(&folder_path) {
                                    Ok(_) => {
                                        // se il caricamento e andato a buon fine
                                        if let Err(e) = player.play_playlist() {
                                            println!(
                                                " ❌ Errore nell'avvio della playlist : {} ",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        println!(
                                            "❌ Errore nel caricamento della playlist : {} ",
                                            e
                                        )
                                    }
                                }
                            }
                            "show" => {
                                player.show_playlist();
                            }
                            "status" => {
                                player.check_playlist_progress();
                                player.show_playlist();
                            }
                            // Correzione: gestire tutti i casi possibili
                            _ => {
                                println!("❌ Sottocomando playlist non riconosciuto. Usa 'run', 'show' o 'status'");
                            }
                        }
                    }

                    //comando non riconosciuto
                    _ => {
                        println!("❌ Comando non riconosciuto. Usa 'help' per vedere i comandi disponibili");
                    }
                }
            }

            // gestisce errore della lettura dell input
            Err(e) => {
                println!("❌ Errore nella lettura dell'input : {}", e);
                break;
            }
        }

        // Controlla periodicamente lo stato della playlist
        if player.playlist_active {
            player.check_playlist_progress();
        }

        // piccola pausa per evitare che il loop consumi troppa cpu
        thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

