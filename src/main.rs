use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, SetBackgroundColor};
use crossterm::terminal::{window_size, WindowSize};
use crossterm::{
    cursor, style::{self, Stylize}, terminal,
    ExecutableCommand,
    QueueableCommand,
};
use palette::{FromColor, Hsl, Srgb};
use rand::distr::{Distribution, Uniform};
use std::io::{self, Stdout, Write};
use std::process;

fn solve_hanoi(disc_config : &mut Vec<(usize, u8)>, stdout: &mut Stdout,
               n: isize, from: u8, to: u8, via: u8, colors: &Vec<Hsl>) -> io::Result<()> {
    if n == -1 {
        return Ok(());
    }
    solve_hanoi(disc_config,stdout,n - 1, from, via, to, colors)?;
    disc_config[n as usize].1 = to;
    loop{
        match read()? {
            Event::Key(KeyEvent{
                code : KeyCode::Enter,..
                       }) => {
                clear_screen(stdout)?;
                print_towers(stdout)?;
                print_discs(stdout, disc_config,colors)?;
                stdout.flush()?;
                break;
            }
            Event::Key(KeyEvent{
                code : KeyCode::Char('c'),
                modifiers,..
                       }) if modifiers == KeyModifiers::CONTROL => {
                cleanup(stdout)?;
                process::exit(1);
            }
            _=>{}
        }
    }
    solve_hanoi(disc_config,stdout,n - 1, via, to, from,colors)?;
    Ok(())
}
fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let number :usize = match std::env::args().nth(1) {
        Some(x)=>{
            match x.parse::<usize>(){
                Ok(x)=>x,
                Err(_)=>{
                    println!("Invalid args. Need one number as argument , that is the number of discs.");
                    return Ok(());
                }
            }
        }
        None => {
            println!("Invalid args. Need one number as argument , that is the number of discs.");
            return Ok(());
        }
    };

    terminal::enable_raw_mode()?;
    clear_screen(&mut stdout)?;
    stdout.execute(cursor::Hide)?;

    let colors = generate_random(number);
    print_towers(&mut stdout)?;
    let mut config : Vec<(usize,u8)> = (0..number).map(|x| (x,1)).collect();
    print_discs(&mut stdout, &mut config, &colors)?;
    stdout.flush()?;
    solve_hanoi(&mut config,&mut stdout,(number-1) as isize,1,3,2,&colors)?;
    print_towers(&mut stdout)?;
    print_discs(&mut stdout, &mut config, &colors)?;
    stdout.flush()?;
    loop{
        match read()? {
            Event::Key(KeyEvent{
                           code : KeyCode::Enter,..
                       }) => {
                break;
            }
            Event::Key(KeyEvent{
                           code : KeyCode::Char('c'),
                           modifiers,..
                       }) if modifiers == KeyModifiers::CONTROL => {
                cleanup(&mut stdout)?;
                process::exit(1);
            }
            _=>{}
        }
    }
    cleanup(&mut stdout)?;
    Ok(())
}

fn print_discs(stdout: &mut Stdout, disc_config: &mut Vec<(usize, u8)>, colors : &Vec<Hsl>) -> io::Result<()> {
    let WindowSize {
        rows,
        columns: cols,
        width: _,
        height: _,
    } = window_size().expect("failed to get window size");
    if disc_config.iter().all(|&x|x.1 == 3) {
        stdout.execute(cursor::MoveTo(0, 1))?;
        println!("SOLVED!!");
    }

    let t = |x: u8| (cols * x as u16) / 4;
    let mut numbers = vec![0;3];
    for i in 0..3u8 {
        numbers[usize::from(i)]= disc_config.iter().filter(|&&x|x.1 ==i+1 ).collect::<Vec<_>>().len();
    }
    let mut towers = vec![0;3];
    for disc  in disc_config {
        for y in t(disc.1) - disc.0 as u16..=t(disc.1) + disc.0 as u16 {
            stdout.queue(cursor::MoveTo(y, rows -( numbers[(disc.1-1) as usize] as u16- towers[disc.1 as usize-1])-1))?;
            let rgb = Srgb::from_color(colors[disc.0]);
            stdout.queue(style::PrintStyledContent("█".with(
                Color::Rgb { r: (rgb.red*255.0) as u8,
                    g:(rgb.green*255.0) as u8,
                    b:(rgb.blue*255.0) as u8
                }
            )))?;
        }
        towers[disc.1 as usize -1] += 1;
    }
    Ok(())
}

fn generate_random(x: usize) -> Vec<Hsl> {
    let mut rng = rand::rng();
    let colors = Uniform::new_inclusive(0.0, 359.99f32)
        .expect("Failed to create uniform distribution: invalid range");
    let mut output = Vec::with_capacity(x);
    let hue = colors.sample(&mut rng);
    for i in 0..x {
        output.push(Hsl::new(hue + 360.0*i as f32/x as f32, 1.0, 0.5));
    }
    output
}

fn print_towers(stdout: &mut Stdout) -> io::Result<()> {
    let WindowSize {
        rows,
        columns: cols,
        width: _,
        height: _,
    } = window_size().expect("failed to get window size");
    stdout.execute(cursor::MoveTo(0, 0))?;
    println!("Press Enter to go to next step. Press Ctrl+C to exit.");
    let t = |x: u8| (cols * x as u16) / 4;

    //2 rows buffer height
    let buffer = 2;
    for x in 1..4 {
        let pos = t(x);
        for y in buffer..rows - 1 {
            stdout.queue(cursor::MoveTo(pos, y))?;
            stdout.queue(style::PrintStyledContent("█".white()))?;
        }
    }
    for x in 0..cols {
        stdout.queue(cursor::MoveTo(x, rows - 1))?;
        stdout.queue(style::PrintStyledContent("█".white()))?;
    }
    Ok(())
}
fn clear_screen(stdout: &mut Stdout) -> io::Result<()> {
    stdout.execute(SetBackgroundColor(Color::Black))?;
    stdout.execute(terminal::Clear(terminal::ClearType::Purge))?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    Ok(())
}
fn cleanup(stdout : &mut Stdout) -> io::Result<()> {
    stdout.queue(terminal::Clear(terminal::ClearType::Purge))?;
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(SetBackgroundColor(Color::Reset))?;
    stdout.queue(cursor::Show)?;
    stdout.queue(cursor::MoveTo(0,0))?;
    stdout.flush()?;
    terminal::disable_raw_mode()?;
    println!();
    println!();
    Ok(())
}