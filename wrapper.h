#ifndef NEEDLE_H
#define NEEDLE_H

#include <stdint.h>

int  needle_load(const char* weights_blob, uint64_t len);
int  needle_init(const char* system, const char* tools_json, const char* tool_index_path);
int  needle_complete(const char* text, int max_new_tokens, char* out_buf, int out_buf_len);
void needle_reset(void);

#endif // NEEDLE_H