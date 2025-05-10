// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#pragma once

#include <cstdint>
#include <limits>

template <typename DERIVED>
struct Mmio {
  const std::size_t addr;
  std::size_t cache;

  constexpr Mmio(std::size_t addr) : addr(addr), cache{0} {}

  void commit() { *(reinterpret_cast<volatile std::size_t*>(addr)) = cache; }

  DERIVED& fetch() {
    cache = *(reinterpret_cast<volatile std::size_t*>(addr));
    return *static_cast<DERIVED*>(this);
  }

  template <typename BITFIELD, std::size_t OFFSET, std::size_t BITS>
  class BitField {
    BITFIELD* const reg;

   public:
    constexpr BitField(BITFIELD* reg) : reg(reg) {}

    constexpr std::size_t mask() {
      static_assert(BITS <= sizeof(std::size_t) * 8);
      if constexpr (BITS == sizeof(std::size_t) * 8) {
        return std::numeric_limits<std::size_t>::max();
      }
      return ((0x01 << BITS) - 1) << OFFSET;
    }

    // This function only exist if BITS > 1
    constexpr BITFIELD& write(const std::size_t value)
      requires(BITS > 1)
    {
      clear();
      reg->cache |= ((value << OFFSET) & mask());
      return *reg;
    }

    // This function only exist if BITS == 1
    constexpr BITFIELD& set()
      requires(BITS == 1)
    {
      reg->cache |= (0x01 << OFFSET);
      return *reg;
    }

    // This function only exist if BITS == 1
    constexpr BITFIELD& reset()
      requires(BITS == 1)
    {
      clear();
      return *reg;
    }

    // This function only exist if BITS == 1
    constexpr bool is_set()
      requires(BITS == 1)
    {
      return (reg->cache & mask()) == mask();
    }

    // This function only exist if BITS > 1
    constexpr std::size_t get()
      requires(BITS > 1)
    {
      return (reg->cache & mask()) >> OFFSET;
    }

    constexpr BITFIELD& clear() {
      reg->cache &= ~mask();
      return *reg;
    }
  };
};
