use std::env;
use std::fs::OpenOptions;
use thiserror::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::process::ExitCode;
use std::path::Path;
use std::io::Read;
use std::io::Write;

#[derive(Error, Debug)]
pub enum RunError{
    Generic(String),
    File(String, std::io::Error),
}

impl Display for RunError{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use RunError::*;
        match self{
            Generic(s) => write!(f, "{}", s),
            File(fname, e) => write!(f, "Error: {}: {}", fname, e),
        }
    }
}

/// Tries to open a file, wraps any errors to include the filename
fn open<P>(oo: &mut OpenOptions, p: P) -> Result<std::fs::File, RunError> where P: AsRef<Path>, P:std::fmt::Display {
    match oo.open(&p){
        Err(e) => Err(RunError::File(p.to_string(), e)),
        Ok(f) => Ok(f),
    }
}

fn main() -> ExitCode {
    let result = (||{
        let mut a = env::args();
        let first = a.next();
        if a.len() != 2{
            let prog = match first{
                Some(prog) => prog,
                None => "scr2bmp".to_string()
            };
            return Err(RunError::Generic(format!("Usage: {} <path to input> <path to output>", prog)));
        }
        let fname_in = a.next().unwrap();
        let fname_out = a.next().unwrap();
        let mut f_in = open(OpenOptions::new().read(true), fname_in)?;
        let mut f_out = open(OpenOptions::new().write(true).create(true).truncate(true), fname_out)?;
        let mut scr = [0;6912];
        match f_in.read_exact(&mut scr){
            Err(e) => Err(RunError::Generic(format!("Error reading input file: {}", e))),
            Ok(()) => Ok(()),
        }?;
        let mut extra = [0;1];
        match f_in.read_exact(&mut extra){
            Err(e) => {
                match e.kind(){
                    std::io::ErrorKind::UnexpectedEof => {/* good */},
                    e => {eprintln!("Non fatal error: Ambiguity near end of input file. What does this mean? I'm not sure, but the exact error message is: {}", e);},
                }
            },
            Ok(()) => {eprintln!("Warning: Extra data at end of input file. Is the file a valid scr?");}
        };
        let write_result: Result<(), std::io::Error> = (||{
            // || = Bitmap header -- = BITMAPINFOHEADER
            
            // || file size 14+40+(192*256/2)+(16*4)=24694
            // || reserved
            // || offset (14+40+(16*4)=118)
            // -- header size (40)
            // -- width (256)
            // -- height (192)
            // -- color planes (1)
            // -- bits per pixel (4)
            // -- compression (none)
            // -- image size (256*192/2=24576)
            // -- horizontal ppm
            // -- vertical ppm
            // -- colours (0=default)
            // -- important colours (0=default)
            // (colour table - 16 entries of 4 bytes each in BGR0 format)
            f_out.write_all(
                b"BM\
                \x76\x60\x00\x00\
                \x00\x00\x00\x00\
                \x76\x00\x00\x00\
                \x28\x00\x00\x00\
                \x00\x01\x00\x00\
                \xc0\x00\x00\x00\
                \x01\x00\
                \x04\x00\
                \x00\x00\x00\x00\
                \x00\x60\x00\x00\
                \x00\x10\x00\x00\
                \x00\x10\x00\x00\
                \x00\x00\x00\x00\
                \x00\x00\x00\x00\
                \x00\x00\x00\x00\
                \xc6\x17\x00\x00\
                \x00\x00\xcd\x00\
                \xc5\x00\xcc\x00\
                \x00\xc4\x00\x00\
                \xc2\xc5\x00\x00\
                \x00\xc0\xc2\x00\
                \xc1\xc1\xc1\x00\
                \x00\x00\x00\x00\
                \xff\x22\x00\x00\
                \x00\x00\xff\x00\
                \xff\x00\xff\x00\
                \x00\xff\x00\x00\
                \xff\xff\x00\x00\
                \x00\xfd\xff\x00\
                \xff\xff\xff\x00\
                "
            )?;
            let bitmap = &scr[0..6144];
            let attrs = &scr[6144..6912];
            for my in 0..192{
                let y = 191-my;
                let ay = (y & (!0x7)) << 2;
                for ax in 0..32{
                    let attr = attrs[ay+ax];
                    let paper = (attr >> 3) & 0xf;
                    let ink = (attr & 0x7) | (paper & 0x8);
                    // the special-sauce address pattern
                    let index = ((y & 0x7) << 3) | ((y & 0x38) >> 3) | (y & 0xc0);
                    let index = (index << 5) + ax;
                    let bits = bitmap[index];
                    for i in 0..4{
                        let nextbits = bits >> (2*(3-i));
                        let b2 = !nextbits;
                        let inkh = ink << 4;
                        let paperh = paper << 4;
                        let pair = ((nextbits & 1) * ink)
                        |(((nextbits & 2)>>1) * inkh)
                        |((b2 & 1) * paper)
                        |(((b2 & 2)>>1) * paperh);
                        f_out.write(&[pair])?;
                    }
                }
            }
            Ok(())
        })();
        match write_result{
            Err(e) => Err(RunError::Generic(format!("Error writing output file: {}", e))),
            Ok(()) => Ok(()),
        }
    })();
    match result{
        Err(ref e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        },
        _=>{ExitCode::SUCCESS},
    }
}
