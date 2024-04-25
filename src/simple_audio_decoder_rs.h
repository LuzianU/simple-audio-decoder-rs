#include <stddef.h>
#include <stdbool.h>

typedef struct
{
    size_t channels;
    size_t frames;
    bool is_done;
    void *buffer;
} CResampleResult;

void *pcm_new_from_file(const char *file);

void *pcm_new_from_data(const void *data, size_t size);

void pcm_free(void *pcm_ptr);

void *audio_clip_new(const void *pcm_pointer, size_t target_sample_rate, size_t chunk_size);

void audio_clip_free(void *audio_clip_ptr);

void *audio_clip_resample_next(void *audio_clip_ptr);

void resample_result_free(void *audio_clip_ptr);