use std::{fs::File, io::Write, slice};

use simple_audio_decoder_rs::{AudioClip, ResampleContinuation};

fn main() {
    let file = "examples/beat.wav";
    let target_sample_rate = 192000;
    let chunk_size = 1024;

    let clip = AudioClip::from_file(file, target_sample_rate, chunk_size);

    if clip.is_none() {
        println!("Failed to create AudioClip from file");
        return;
    }

    let mut clip = clip.unwrap();

    let mut resampled = Vec::new();

    loop {
        match clip.resample_next() {
            Ok((buffer, resample_result)) => {
                if resampled.is_empty() {
                    resampled = buffer.to_vec();
                } else {
                    (0..resampled.len()).for_each(|i| {
                        resampled[i].extend(buffer[i].iter());
                    });
                }

                if let ResampleContinuation::NoMoreData = resample_result {
                    break;
                }
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        }
    }

    // optinally clear cache
    simple_audio_decoder_rs::clear_cache();

    // interleave channels
    let mut interleaved = Vec::new();
    for i in 0..resampled[0].len() {
        (0..resampled.len()).for_each(|j| {
            interleaved.push(resampled[j][i]);
        });
    }

    // write interleaved data to file
    let mut output = File::create("interleaved.dat").unwrap();
    output
        .write_all(unsafe {
            slice::from_raw_parts(
                interleaved.as_ptr() as *const u8,
                interleaved.len() * std::mem::size_of::<f32>(),
            )
        })
        .unwrap();

    println!("Wrote interleaved data to interleaved.dat");
}
