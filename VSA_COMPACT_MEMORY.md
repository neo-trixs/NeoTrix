# VSA Compact Memory Migration Plan

## Current state
- Type: `Vec<u8>` (24 bytes heap overhead per vector)
- Allocation sites: ~151 (vec![0u8; VSA_DIM], vec![1u8; VSA_DIM])
- Return types: 99 `-> Vec<u8>` in nt_core_hcube/

## Target state
- Type alias: `pub type VsaVec = Box<[u8; VSA_BYTES]>;` (8 bytes, no capacity field)
- VSA_BYTES = 512 (4096 bits / 8)

## Migration phases

### Phase 1: Type alias + QuantizedVSA change (safe, no callers affected)
- Add `pub type VsaVec = Box<[u8; VSA_BYTES]>;` to vsa_quantized.rs
- Change QuantizedVSA internal helpers to return VsaVec
- Keep old `-> Vec<u8>` methods as wrappers calling new `-> VsaVec` methods

### Phase 2: Core hcube migration (risky, 27 files)
- Change all 99 return types in nt_core_hcube/ from `-> Vec<u8>` to `-> VsaVec`
- Uses `Box::new([0u8; VSA_BYTES])` for zero vectors
- Update test assertions to compare slices: `assert_eq!(&*v, &expected[..])`

### Phase 3: External callers (safe, mechanical)
- All `&[u8]` params already accept Box<[u8; N]> via Deref
- NeValue::Vsa(Vec<u8>) → NeValue::Vsa(VsaVec)
- Fix eval.rs pattern matches

## Key files
- vsa_quantized.rs: ~500 lines core
- eval.rs: ~50 pattern matches on NeValue::Vsa
- hcube/: ~27 files, 99 return types, 151 alloc sites

## Safety
- All methods take `&[u8]` — No signature changes at call sites
- `Box<[u8; VSA_BYTES]>` is Send + Sync (unlike Vec)
- Serialization needs custom impl or through `&[u8]`
