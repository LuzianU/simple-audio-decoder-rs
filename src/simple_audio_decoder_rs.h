#include <stddef.h>
#include <stdbool.h>

struct CResampleResult
{
    size_t channels;
    size_t frames;
    bool is_done;
    void *buffer;
};

void *audio_clip_from_file(const char *file, size_t target_sample_rate, size_t chunk_size);

void audio_clip_free(void *audio_clip_ptr);

void *audio_clip_resample_next(void *audio_clip_ptr);