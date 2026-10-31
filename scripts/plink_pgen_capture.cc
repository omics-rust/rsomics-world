// Test-only bridge to PLINK pgenlib cc2c95e61c509d49aaf884d2ec0956a921770745.
// Upstream pgenlib is LGPL-3.0-or-later; retain its COPYING.LESSER and notices.
#include "pgenlib_read.h"
#include "pgenlib_ffi_support.h"

#include <cstring>
#include <algorithm>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

using namespace plink2;

struct Reader {
  PgenFileInfo info;
  PgenReader reader;
  std::vector<void*> allocations;
  Reader() { PreinitPgfi(&info); PreinitPgr(&reader); }
  template <typename T> T* alloc(size_t count) {
    void* ptr = nullptr;
    const size_t bytes = ((count * sizeof(T) + kCacheline - 1) / kCacheline) * kCacheline;
    if (cachealigned_malloc(bytes ? bytes : kCacheline, &ptr)) throw std::bad_alloc();
    allocations.push_back(ptr);
    std::memset(ptr, 0, bytes ? bytes : kCacheline);
    return static_cast<T*>(ptr);
  }
  ~Reader() {
    PglErr error = kPglRetSuccess;
    CleanupPgr(&reader, &error);
    CleanupPgfi(&info, &error);
    for (auto* ptr : allocations) aligned_free(ptr);
  }
};

static void require(PglErr error, const char* operation) {
  if (error != kPglRetSuccess) throw std::runtime_error(std::string(operation) + " error=" + std::to_string(static_cast<uint32_t>(error)));
}

static bool bit(const uintptr_t* values, uint32_t index) {
  return (values[index / kBitsPerWord] >> (index % kBitsPerWord)) & 1;
}

static std::vector<std::vector<std::string>> rows(const char* path) {
  std::ifstream stream(path);
  if (!stream) throw std::runtime_error("cannot open metadata");
  std::string line;
  std::vector<std::vector<std::string>> result;
  while (std::getline(stream, line)) {
    if (line.empty() || line[0] == '#') continue;
    std::istringstream input(line);
    std::vector<std::string> fields;
    for (std::string value; input >> value;) fields.push_back(value);
    if (!fields.empty()) result.push_back(fields);
  }
  if (!stream.eof()) throw std::runtime_error("metadata read failure");
  return result;
}

int main(int argc, char** argv) {
  try {
    if (argc != 4) throw std::runtime_error("usage: plink_pgen_capture data.pgen data.pvar data.psam");
    auto variants = rows(argv[2]);
    auto samples = rows(argv[3]);
    if (variants.empty() || samples.empty() || variants.size() > 1000 || samples.size() > 1000)
      throw std::runtime_error("compact capture shape invalid");
    const uint32_t nv = static_cast<uint32_t>(variants.size());
    const uint32_t ns = static_cast<uint32_t>(samples.size());
    Reader state;
    PgenHeaderCtrl ctrl;
    uintptr_t phase1_size = 0;
    char error[kPglErrstrBufBlen] = {};
    const PglErr phase1 = PgfiInitPhase1(argv[1], nullptr, nv, ns, &ctrl, &state.info, &phase1_size, error);
    if (phase1 != kPglRetSuccess) throw std::runtime_error(error);
    state.info.allele_idx_offsets = state.alloc<uintptr_t>(nv + 1);
    uint32_t max_alleles = 2;
    for (uint32_t v = 0; v != nv; ++v) {
      if (variants[v].size() < 5) throw std::runtime_error("short PVAR row");
      const auto& alts = variants[v][4];
      uint32_t nalleles = 2;
      for (char c : alts) nalleles += c == ',';
      max_alleles = std::max(max_alleles, nalleles);
      state.info.allele_idx_offsets[v + 1] = state.info.allele_idx_offsets[v] + nalleles;
    }
    state.info.max_allele_ct = max_alleles;
    if ((ctrl & 0xc0) == 0xc0) state.info.nonref_flags = state.alloc<uintptr_t>(BitCtToWordCt(nv));
    auto* phase1_alloc = state.alloc<unsigned char>(phase1_size * kCacheline);
    uintptr_t reader_size = 0;
    uint32_t max_width = 0;
    const PglErr phase2 = PgfiInitPhase2(ctrl, 1, 0, 0, 0, nv, &max_width, &state.info, phase1_alloc, &reader_size, error);
    if (phase2 != kPglRetSuccess) throw std::runtime_error(error);
    require(PgrInit(argv[1], max_width, &state.info, &state.reader, state.alloc<unsigned char>(reader_size * kCacheline)), "PgrInit");
    PgrSampleSubsetIndex subset;
    PgrClearSampleSubsetIndex(&state.reader, &subset);
    PgenVariant value;
    PreinitPgv(&value);
    const size_t words = BitCtToWordCt(ns);
    value.genovec = state.alloc<uintptr_t>(NypCtToVecCt(ns) * kWordsPerVec);
    value.patch_01_set = state.alloc<uintptr_t>(words);
    value.patch_10_set = state.alloc<uintptr_t>(words);
    value.patch_01_vals = state.alloc<AlleleCode>(ns);
    value.patch_10_vals = state.alloc<AlleleCode>(ns * 2);
    value.phasepresent = state.alloc<uintptr_t>(words);
    value.phaseinfo = state.alloc<uintptr_t>(words);
    value.dosage_present = state.alloc<uintptr_t>(words);
    value.dosage_main = state.alloc<uint16_t>(ns);
    value.dphase_present = state.alloc<uintptr_t>(words);
    value.dphase_delta = state.alloc<int16_t>(ns);
    auto* allele_codes = state.alloc<int32_t>(ns * 2);
    auto* phases = state.alloc<unsigned char>(ns);
    auto* pcheck = state.alloc<uintptr_t>(words);
    auto* picheck = state.alloc<uintptr_t>(words);
    auto* dcheck = state.alloc<uintptr_t>(words);
    auto* dvalues = state.alloc<uint16_t>(ns);
    auto* gcheck = state.alloc<uintptr_t>(NypCtToVecCt(ns) * kWordsPerVec);
    std::cout << "{\"schema\":1,\"samples\":" << ns << ",\"variants\":" << nv
              << ",\"source\":\"cc2c95e61c509d49aaf884d2ec0956a921770745\",\"semantics\":\"raw codec; chromosome correction belongs to caller\"}\n";
    for (uint32_t v = 0; v != nv; ++v) {
      for (auto* mask : {value.phasepresent, value.phaseinfo, value.dosage_present, value.dphase_present})
        std::memset(mask, 0, words * sizeof(uintptr_t));
      value.dosage_ct = value.dphase_ct = 0;
      value.patch_01_ct = value.patch_10_ct = 0;
      const uint32_t alleles = state.info.allele_idx_offsets[v+1] - state.info.allele_idx_offsets[v];
      if (alleles > 2) {
        require(PgrGetMP(nullptr, subset, ns, v, &state.reader, &value), "PgrGetMP");
      } else {
        require(PgrGetDp(nullptr, subset, ns, v, &state.reader, &value), "PgrGetDp");
        uint32_t nphase = 0, ndose = 0;
        std::memset(pcheck, 0, words * sizeof(uintptr_t));
        std::memset(dcheck, 0, words * sizeof(uintptr_t));
        require(PgrGetP(nullptr, subset, ns, v, &state.reader, gcheck, pcheck, picheck, &nphase), "PgrGetP");
        require(PgrGetD(nullptr, subset, ns, v, &state.reader, gcheck, dcheck, dvalues, &ndose), "PgrGetD");
        if (nphase != value.phasepresent_ct || ndose != value.dosage_ct)
          throw std::runtime_error("P/D/Dp presence count disagreement");
        for (uint32_t s = 0; s != ns; ++s) {
          if (bit(pcheck, s) != bit(value.phasepresent, s) || bit(dcheck, s) != bit(value.dosage_present, s))
            throw std::runtime_error("P/D/Dp presence mask disagreement");
        }
        for (uint32_t d = 0; d != ndose; ++d)
          if (dvalues[d] != value.dosage_main[d]) throw std::runtime_error("D/Dp dosage disagreement");
      }
      GenoarrMPToAlleleCodesMinus9(&value, ns, phases, allele_codes);
      const bool nonref = state.info.nonref_flags ? bit(state.info.nonref_flags, v) : ((state.info.gflags & kfPgenGlobalAllNonref) != 0);
      std::cout << "{\"variant\":" << v << ",\"provisional_ref\":" << (nonref ? "true" : "false") << ",\"calls\":[";
      uint32_t dose_index = 0, dphase_index = 0;
      for (uint32_t s = 0; s != ns; ++s) {
        if (s) std::cout << ',';
        const bool phase = value.phasepresent_ct && bit(value.phasepresent, s);
        const bool dose = value.dosage_ct && bit(value.dosage_present, s);
        const bool dp = value.dphase_ct && bit(value.dphase_present, s);
        std::cout << '[' << allele_codes[2*s] << ',' << allele_codes[2*s+1] << ',' << (phase ? 1 : 0) << ','
                  << (phase && bit(value.phaseinfo, s) ? 1 : 0) << ',';
        if (dose) std::cout << value.dosage_main[dose_index++]; else std::cout << "null";
        std::cout << ',';
        if (dp) std::cout << value.dphase_delta[dphase_index++]; else std::cout << "null";
        std::cout << ']';
      }
      std::cout << "]}\n";
    }
    std::cout.flush();
    if (!std::cout) throw std::runtime_error("capture output IO failure");
    return 0;
  } catch (const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
