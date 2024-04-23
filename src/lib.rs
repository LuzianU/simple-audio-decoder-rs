use std::{fs::File, io::BufReader, time::Duration};

use rodio::{Decoder, Source};
use rubato::Resampler;

mod ffi;

pub struct AudioClip {
    pcm: Pcm,
    resampler: rubato::FastFixedOut<f32>,
    buffer_in: Vec<Vec<f32>>,
    buffer_out: Vec<Vec<f32>>,
    index: usize,
}

pub struct Pcm {
    data: Vec<f32>,
    sample_rate: usize,
    channels: usize,
}

#[derive(Debug)]
pub enum ResampleContinuation {
    MoreData,
    NoMoreData,
}

impl AudioClip {
    fn new(
        pcm: Pcm,
        resampler: rubato::FastFixedOut<f32>,
        buffer_in: Vec<Vec<f32>>,
        buffer_out: Vec<Vec<f32>>,
    ) -> Self {
        Self {
            pcm,
            resampler,
            buffer_in,
            buffer_out,
            index: 0,
        }
    }

    pub fn resample_next(
        &mut self,
    ) -> Result<(&Vec<Vec<f32>>, ResampleContinuation), rubato::ResampleError> {
        let input_frames_next = self.resampler.input_frames_next();

        let input_frames_next =
            input_frames_next.min((self.pcm.data.len() - self.index) / self.pcm.channels);

        if input_frames_next == 0 {
            return Ok((&self.buffer_out, ResampleContinuation::NoMoreData));
        }

        // pad the rest of the buffer with zeros
        self.buffer_in.iter_mut().for_each(|buffer| {
            (input_frames_next..self.resampler.input_frames_next()).for_each(|i| {
                buffer[i] = 0.0;
            });
        });

        // copy currents batch input data to buffer_in
        let mut i = 0;
        while i < input_frames_next {
            for c in 0..self.pcm.channels {
                self.buffer_in[c][i] = self.pcm.data[self.index + i * self.pcm.channels + c];
            }
            i += 1;
        }

        self.index += input_frames_next * self.pcm.channels;

        let _ = self
            .resampler
            .process_into_buffer(&self.buffer_in, &mut self.buffer_out, None)?;

        // ignore process_into_buffer result's (read, written) since it is always equal to input_frames_next and output_frames_next

        if self.index >= self.pcm.data.len() {
            Ok((&self.buffer_out, ResampleContinuation::NoMoreData))
        } else {
            Ok((&self.buffer_out, ResampleContinuation::MoreData))
        }
    }

    pub fn from_file(file: &str, target_sample_rate: usize, chunk_size: usize) -> Option<Self> {
        if let Some(helper) = SampleConvertHelper::new(file) {
            let sample_rate = helper.sample_rate() as usize;
            let channels = helper.channels() as usize;

            let data: Vec<f32> = helper.convert_samples().collect();

            let pcm = Pcm {
                data,
                sample_rate,
                channels,
            };

            let resample_ratio = target_sample_rate as f64 / sample_rate as f64;

            let resampler = rubato::FastFixedOut::<f32>::new(
                resample_ratio,
                1.0,
                rubato::PolynomialDegree::Septic, // quality
                chunk_size,
                channels,
            )
            .ok()?;

            let buffer_in = resampler.input_buffer_allocate(true);
            let buffer_out = resampler.output_buffer_allocate(true);

            return Some(Self::new(pcm, resampler, buffer_in, buffer_out));
        }
        None
    }
}

struct SampleConvertHelper {
    decoder: Decoder<BufReader<File>>,
}

impl SampleConvertHelper {
    pub fn new(file: &str) -> Option<Self> {
        if let Ok(file) = std::fs::File::open(file) {
            if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                return Some(SampleConvertHelper { decoder });
            }
        }
        None
    }
}
impl Iterator for SampleConvertHelper {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next()
    }
}

impl Source for SampleConvertHelper {
    fn current_frame_len(&self) -> Option<usize> {
        self.decoder.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.decoder.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.decoder.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.decoder.total_duration()
    }
}
