// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#pragma once

#include <cstdint>
#include <limits>
namespace reismmio {
  struct Register{
    const std::size_t addr = 0;
    std::size_t cache = 0;

    inline void commit() { *(reinterpret_cast<volatile std::size_t*>(addr)) = cache; }
    inline void fetch() {
      cache = *(reinterpret_cast<volatile std::size_t*>(addr));
    }
  };

  template <std::size_t OFFSET, std::size_t BITS>
    class BitField {
      Register reg{0};

      public:

      static consteval std::size_t mask() {
        static_assert(BITS <= sizeof(std::size_t) * 8);
        if constexpr (BITS == sizeof(std::size_t) * 8) {
          return std::numeric_limits<std::size_t>::max();
        }
        return ((0x01 << BITS) - 1) << OFFSET;
      }

      static consteval std::size_t max() {
        return (1 << BITS) - 1;
      }

      // This function only exist if BITS > 1
      inline constexpr auto& write(const std::size_t value)
        requires(BITS > 1)
        {
          clear();
          reg.cache |= ((value << OFFSET) & mask());
          return *this;
        }

      // This function only exist if BITS == 1
      inline constexpr auto& set()
        requires(BITS == 1)
        {
          reg.cache |= (0x01 << OFFSET);
          return *this;
        }

      // This function only exist if BITS == 1
      inline constexpr auto& reset()
        requires(BITS == 1)
        {
          clear();
          return *this;
        }

      // This function only exist if BITS == 1
      inline constexpr auto& toggle()
        requires(BITS == 1)
        {
          reg.cache ^= (0x01 << OFFSET);
          return *this;
        }

      // This function only exist if BITS > 1
      inline constexpr auto& bit_mask(std::size_t value, std::size_t offset)
        requires(BITS > 1)
        {
          reg.cache &= ~((0x01 << (OFFSET+offset)) & mask());
          reg.cache |= ((value << (OFFSET+offset)) & mask());
          return *this;
        }

      // This function only exist if BITS == 1
      inline constexpr auto& bit(bool bit)
        requires(BITS == 1)
        {
          reg.cache |= (static_cast<std::size_t>(bit) << OFFSET);
          return *this;
        }

      // This function only exist if BITS == 1
      inline constexpr bool is_set()
        requires(BITS == 1)
        {
          return (reg.cache & mask()) == mask();
        }

      // This function only exist if BITS > 1
      inline constexpr std::size_t get()
        requires(BITS > 1)
        {
          return (reg.cache & mask()) >> OFFSET;
        }

      inline constexpr auto& clear() {
        reg.cache &= ~mask();
        return *this;
      }

      inline void commit() { reg.commit(); }
    };
};
