use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek},
    time::Duration,
};

use rodio::{Decoder, Source};
use rubato::Resampler;

mod ffi;

pub struct AudioClip<'a> {
    pcm: &'a Pcm,
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

impl Pcm {
    pub fn new_from_file(file: &str) -> Option<Self> {
        if let Some(helper) = SampleConvertHelper::new_from_file(file) {
            let sample_rate = helper.sample_rate() as usize;
            let channels = helper.channels() as usize;

            let data: Vec<f32> = helper.convert_samples().collect();

            let pcm = Pcm {
                data,
                sample_rate,
                channels,
            };

            return Some(pcm);
        }

        None
    }

    pub fn new_from_data(data: Vec<u8>) -> Option<Self> {
        if let Some(helper) = SampleConvertHelper::new_from_data(data) {
            let sample_rate = helper.sample_rate() as usize;
            let channels = helper.channels() as usize;

            let data: Vec<f32> = helper.convert_samples().collect();

            let pcm = Pcm {
                data,
                sample_rate,
                channels,
            };

            return Some(pcm);
        }

        None
    }
}

impl<'a> AudioClip<'a> {
    pub fn new(pcm: &'a Pcm, target_sample_rate: usize, chunk_size: usize) -> Option<Self> {
        let resample_ratio = target_sample_rate as f64 / pcm.sample_rate as f64;

        let resampler = rubato::FastFixedOut::<f32>::new(
            resample_ratio,
            1.0,
            rubato::PolynomialDegree::Septic, // quality
            chunk_size,
            pcm.channels,
        )
        .ok()?;

        let buffer_in = resampler.input_buffer_allocate(true);
        let buffer_out = resampler.output_buffer_allocate(true);

        Some(Self {
            pcm,
            resampler,
            buffer_in,
            buffer_out,
            index: 0,
        })
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
}

pub struct SampleConvertHelper<R>
where
    R: Read + Seek + Send + Sync + 'static,
{
    decoder: Decoder<R>,
}

impl SampleConvertHelper<BufReader<File>> {
    pub fn new_from_file(file: &str) -> Option<Self> {
        if let Ok(file) = std::fs::File::open(file) {
            return SampleConvertHelper::new(BufReader::new(file));
        }
        None
    }
}

impl SampleConvertHelper<Cursor<Vec<u8>>> {
    pub fn new_from_data(data: Vec<u8>) -> Option<Self> {
        SampleConvertHelper::new(Cursor::new(data))
    }
}

impl<R> SampleConvertHelper<R>
where
    R: Read + Seek + Send + Sync + 'static,
{
    pub fn new(data: R) -> Option<Self> {
        if let Ok(decoder) = Decoder::new(data) {
            return Some(SampleConvertHelper { decoder });
        }
        None
    }
}
impl<R> Iterator for SampleConvertHelper<R>
where
    R: Read + Seek + Send + Sync + 'static,
{
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next()
    }
}

impl<R> Source for SampleConvertHelper<R>
where
    R: Read + Seek + Send + Sync + 'static,
{
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
