#include <stddef.h>
#include <stdbool.h>

typedef struct
{
    size_t channels;
    size_t frames;
    bool is_done;
    void *buffer;
} CResampleResult;

void clear_cache();

void *audio_clip_from_file(const char *file, size_t target_sample_rate, size_t chunk_size);

void audio_clip_free(void *audio_clip_ptr);

void *audio_clip_resample_next(void *audio_clip_ptr);

void resample_result_free(void *audio_clip_ptr);