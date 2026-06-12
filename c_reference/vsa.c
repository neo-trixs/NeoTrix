#include <stdint.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <math.h>

#define VSA_BYTES 64
#define VSA_BITS  (VSA_BYTES * 8)
#define FFT_N     128

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

typedef struct { double re, im; } Complex;

static Complex c_add(Complex a, Complex b) { Complex r = {a.re + b.re, a.im + b.im}; return r; }
static Complex c_sub(Complex a, Complex b) { Complex r = {a.re - b.re, a.im - b.im}; return r; }
static Complex c_mul(Complex a, Complex b) {
    Complex r = {a.re * b.re - a.im * b.im, a.re * b.im + a.im * b.re};
    return r;
}
static Complex c_conj(Complex a) { Complex r = {a.re, -a.im}; return r; }

/* Cooley-Tukey radix-2 FFT in-place (n must be power of 2) */
static void fft_inplace(Complex *x, int n) {
    /* bit-reversal permutation */
    for (int i = 1, j = 0; i < n; i++) {
        int bit = n >> 1;
        for (; j & bit; bit >>= 1) j ^= bit;
        j ^= bit;
        if (i < j) { Complex t = x[i]; x[i] = x[j]; x[j] = t; }
    }
    /* iterative butterfly */
    for (int len = 2; len <= n; len <<= 1) {
        double ang = -2.0 * M_PI / len;
        Complex wlen = {cos(ang), sin(ang)};
        for (int i = 0; i < n; i += len) {
            Complex w = {1.0, 0.0};
            int half = len >> 1;
            for (int j = 0; j < half; j++) {
                Complex u = x[i + j];
                Complex v = c_mul(w, x[i + j + half]);
                x[i + j] = c_add(u, v);
                x[i + j + half] = c_sub(u, v);
                w = c_mul(w, wlen);
            }
        }
    }
}

/* IFFT in-place: conjugate, FFT, conjugate + divide by n */
static void ifft_inplace(Complex *x, int n) {
    for (int i = 0; i < n; i++) x[i].im = -x[i].im;
    fft_inplace(x, n);
    for (int i = 0; i < n; i++) {
        x[i].im = -(x[i].im / n);
        x[i].re /= n;
    }
}

/* utility: print VSA vector as hex */
static void print_vec(const uint8_t v[VSA_BYTES]) {
    for (int i = 0; i < VSA_BYTES; i++) printf("%02x", v[i]);
    printf("\n");
}

/* utility: parse hex string into VSA vector */
static int parse_vec(const char *hex, uint8_t out[VSA_BYTES]) {
    size_t len = strlen(hex);
    if (len != VSA_BYTES * 2) return -1;
    for (int i = 0; i < VSA_BYTES; i++) {
        char buf[3] = {hex[2*i], hex[2*i+1], 0};
        out[i] = (uint8_t)strtol(buf, NULL, 16);
    }
    return 0;
}

/* normalize byte [0,255] → double [-1, 1]; binary {0,1} maps to {-1,1} */
static double norm_byte(uint8_t b) {
    switch (b) {
        case 0:   return -1.0;
        case 1:   return  1.0;
        default:  return (b / 127.5) - 1.0;
    }
}

/* bound double back to binary {0,1} — match Rust f64_to_binary */
static uint8_t unorm_double(double d) {
    double clamped = (d > 1.0) ? 1.0 : (d < -1.0) ? -1.0 : d;
    int u8val = (int)((clamped + 1.0) * 127.5);
    return (u8val >= 128) ? 1 : 0;
}

/* 1a. bind: FFT-HRR circular convolution — IFFT(FFT(a) * FFT(b)) */
static void op_bind(const uint8_t a[VSA_BYTES], const uint8_t b[VSA_BYTES],
                     uint8_t out[VSA_BYTES]) {
    Complex fa[FFT_N], fb[FFT_N], fc[FFT_N];

    for (int i = 0; i < VSA_BYTES; i++) {
        fa[i].re = norm_byte(a[i]); fa[i].im = 0.0;
        fb[i].re = norm_byte(b[i]); fb[i].im = 0.0;
    }
    for (int i = VSA_BYTES; i < FFT_N; i++) {
        fa[i].re = 0.0; fa[i].im = 0.0;
        fb[i].re = 0.0; fb[i].im = 0.0;
    }

    fft_inplace(fa, FFT_N);
    fft_inplace(fb, FFT_N);

    for (int i = 0; i < FFT_N; i++) fc[i] = c_mul(fa[i], fb[i]);

    ifft_inplace(fc, FFT_N);

    for (int i = 0; i < VSA_BYTES; i++) out[i] = unorm_double(fc[i].re);
}

/* 1b. unbind: IFFT(FFT(c) * conj(FFT(a))) — recovers b approximately */
static void op_unbind(const uint8_t c[VSA_BYTES], const uint8_t a[VSA_BYTES],
                       uint8_t out[VSA_BYTES]) {
    Complex fc[FFT_N], fa[FFT_N];

    for (int i = 0; i < VSA_BYTES; i++) {
        fc[i].re = norm_byte(c[i]); fc[i].im = 0.0;
        fa[i].re = norm_byte(a[i]); fa[i].im = 0.0;
    }
    for (int i = VSA_BYTES; i < FFT_N; i++) {
        fc[i].re = 0.0; fc[i].im = 0.0;
        fa[i].re = 0.0; fa[i].im = 0.0;
    }

    fft_inplace(fc, FFT_N);
    fft_inplace(fa, FFT_N);

    for (int i = 0; i < FFT_N; i++) fc[i] = c_mul(fc[i], c_conj(fa[i]));

    ifft_inplace(fc, FFT_N);

    for (int i = 0; i < VSA_BYTES; i++) out[i] = unorm_double(fc[i].re);
}

/* 1c. xor_bind: original XOR binding (kept for reference) */
static void op_xor_bind(const uint8_t a[VSA_BYTES], const uint8_t b[VSA_BYTES],
                         uint8_t out[VSA_BYTES]) {
    for (int i = 0; i < VSA_BYTES; i++) out[i] = a[i] ^ b[i];
}

/* 2. bundle: per-byte majority (sum >= 128 → 0xFF else 0x00) */
static void op_bundle(const uint8_t *vecs[], int n, uint8_t out[VSA_BYTES]) {
    for (int i = 0; i < VSA_BYTES; i++) {
        uint32_t sum = 0;
        for (int j = 0; j < n; j++) sum += vecs[j][i];
        out[i] = (sum >= 128) ? 0xFF : 0x00;
    }
}

/* 3. permute: cyclic left shift by k bytes */
static void op_permute(const uint8_t a[VSA_BYTES], int k,
                        uint8_t out[VSA_BYTES]) {
    k = ((k % VSA_BYTES) + VSA_BYTES) % VSA_BYTES;
    memcpy(out, a + k, VSA_BYTES - k);
    memcpy(out + VSA_BYTES - k, a, k);
}

/* 4. negate: bitwise NOT */
static void op_negate(const uint8_t a[VSA_BYTES], uint8_t out[VSA_BYTES]) {
    for (int i = 0; i < VSA_BYTES; i++) out[i] = ~a[i];
}

/* 5. similarity: normalized Hamming [0,1] 1 = identical */
static double op_similarity(const uint8_t a[VSA_BYTES], const uint8_t b[VSA_BYTES]) {
    uint32_t diff = 0;
    for (int i = 0; i < VSA_BYTES; i++) {
        uint8_t x = a[i] ^ b[i];
        while (x) { diff += x & 1; x >>= 1; }
    }
    return 1.0 - (double)diff / (double)VSA_BITS;
}

/* 6. cosine: raw cosine similarity on byte vectors */
static double op_cosine(const uint8_t a[VSA_BYTES], const uint8_t b[VSA_BYTES]) {
    double dot = 0.0, na = 0.0, nb = 0.0;
    for (int i = 0; i < VSA_BYTES; i++) {
        dot += (double)a[i] * (double)b[i];
        na  += (double)a[i] * (double)a[i];
        nb  += (double)b[i] * (double)b[i];
    }
    na = sqrt(na); nb = sqrt(nb);
    return (na * nb == 0.0) ? 0.0 : dot / (na * nb);
}

/* 7. hamming_distance: raw bit count */
static uint32_t op_hamming(const uint8_t a[VSA_BYTES], const uint8_t b[VSA_BYTES]) {
    uint32_t diff = 0;
    for (int i = 0; i < VSA_BYTES; i++) {
        uint8_t x = a[i] ^ b[i];
        while (x) { diff += x & 1; x >>= 1; }
    }
    return diff;
}

/* 8. random_vector: xorshift64 deterministic PRNG */
static uint64_t xorshift64(uint64_t *s) {
    uint64_t x = *s;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *s = x;
    return x;
}

static void op_random(uint8_t out[VSA_BYTES], uint64_t seed) {
    for (int i = 0; i < VSA_BYTES; i++)
        out[i] = (uint8_t)(xorshift64(&seed) & 0xFF);
}

/* hex_byte: write a double as 8-byte little-endian hex */
static void print_double(double d) {
    uint64_t bits;
    memcpy(&bits, &d, 8);
    for (int i = 0; i < 8; i++)
        printf("%02x", (uint8_t)(bits >> (i * 8)));
    printf("\n");
}

static void print_u32(uint32_t v) {
    for (int i = 0; i < 4; i++)
        printf("%02x", (uint8_t)(v >> (i * 8)));
    printf("\n");
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <op> [args...]\n", argv[0]);
        fprintf(stderr, "Ops: bind, unbind, xor_bind, bundle, permute, negate, similarity,\n");
        fprintf(stderr, "     cosine, hamming_distance, random_vector\n");
        return 1;
    }

    const char *op = argv[1];
    uint8_t a[VSA_BYTES], b[VSA_BYTES], out[VSA_BYTES];
    int n;

    if (strcmp(op, "bind") == 0) {
        if (argc != 4) { fprintf(stderr, "bind <hex_a> <hex_b>\n"); return 1; }
        if (parse_vec(argv[2], a) || parse_vec(argv[3], b)) return 1;
        op_bind(a, b, out);
        print_vec(out);
    } else if (strcmp(op, "unbind") == 0) {
        if (argc != 4) { fprintf(stderr, "unbind <hex_c> <hex_a>\n"); return 1; }
        if (parse_vec(argv[2], a) || parse_vec(argv[3], b)) return 1;
        op_unbind(a, b, out);
        print_vec(out);
    } else if (strcmp(op, "xor_bind") == 0) {
        if (argc != 4) { fprintf(stderr, "xor_bind <hex_a> <hex_b>\n"); return 1; }
        if (parse_vec(argv[2], a) || parse_vec(argv[3], b)) return 1;
        op_xor_bind(a, b, out);
        print_vec(out);
    } else if (strcmp(op, "bundle") == 0) {
        if (argc < 4) { fprintf(stderr, "bundle <hex_a> <hex_b> [hex_c ...]\n"); return 1; }
        n = argc - 2;
        const uint8_t **vecs = malloc(n * sizeof(uint8_t*));
        for (int i = 0; i < n; i++) {
            vecs[i] = malloc(VSA_BYTES);
            if (parse_vec(argv[2 + i], (uint8_t*)vecs[i])) return 1;
        }
        op_bundle(vecs, n, out);
        print_vec(out);
        for (int i = 0; i < n; i++) free((void*)vecs[i]);
        free(vecs);
    } else if (strcmp(op, "permute") == 0) {
        if (argc != 4) { fprintf(stderr, "permute <hex_a> <k>\n"); return 1; }
        if (parse_vec(argv[2], a)) return 1;
        int k = atoi(argv[3]);
        op_permute(a, k, out);
        print_vec(out);
    } else if (strcmp(op, "negate") == 0) {
        if (argc != 3) { fprintf(stderr, "negate <hex_a>\n"); return 1; }
        if (parse_vec(argv[2], a)) return 1;
        op_negate(a, out);
        print_vec(out);
    } else if (strcmp(op, "similarity") == 0) {
        if (argc != 4) { fprintf(stderr, "similarity <hex_a> <hex_b>\n"); return 1; }
        if (parse_vec(argv[2], a) || parse_vec(argv[3], b)) return 1;
        double s = op_similarity(a, b);
        printf("%.10f\n", s);
    } else if (strcmp(op, "cosine") == 0) {
        if (argc != 4) { fprintf(stderr, "cosine <hex_a> <hex_b>\n"); return 1; }
        if (parse_vec(argv[2], a) || parse_vec(argv[3], b)) return 1;
        double c = op_cosine(a, b);
        printf("%.10f\n", c);
    } else if (strcmp(op, "hamming_distance") == 0) {
        if (argc != 4) { fprintf(stderr, "hamming_distance <hex_a> <hex_b>\n"); return 1; }
        if (parse_vec(argv[2], a) || parse_vec(argv[3], b)) return 1;
        uint32_t h = op_hamming(a, b);
        printf("%u\n", h);
    } else if (strcmp(op, "random_vector") == 0) {
        uint64_t seed = (argc > 2) ? (uint64_t)atoll(argv[2]) : 42;
        op_random(out, seed);
        print_vec(out);
    } else {
        fprintf(stderr, "Unknown op: %s\n", op);
        return 1;
    }
    return 0;
}
