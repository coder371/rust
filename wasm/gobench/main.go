//go:build js && wasm

// The same four workloads as src/bench.rs, statement for statement, exposed to
// the page as `goBench`. Each returns a checksum so the three runtimes can be
// proved to have done identical work.
package main

import "syscall/js"

func matmul(n int) float64 {
	a := make([]float64, n*n)
	b := make([]float64, n*n)
	for i := 0; i < n*n; i++ {
		a[i] = float64(i%17)*0.125 - 1.0
		b[i] = float64(i%23)*0.0625 - 0.5
	}

	c := make([]float64, n*n)
	for i := 0; i < n; i++ {
		for k := 0; k < n; k++ {
			aik := a[i*n+k]
			if aik == 0 {
				continue
			}
			for j := 0; j < n; j++ {
				c[i*n+j] += aik * b[k*n+j]
			}
		}
	}

	sum := 0.0
	for _, v := range c {
		sum += v
	}
	return sum
}

func sieve(limit int) float64 {
	if limit < 2 {
		return 0
	}
	// Go's `int` is 64 bits on this target, so i*i cannot wrap the way it
	// does under Rust's 32-bit usize.
	composite := make([]bool, limit)
	count := 0
	for i := 2; i < limit; i++ {
		if !composite[i] {
			count++
			for j := i * i; j < limit; j += i {
				composite[j] = true
			}
		}
	}
	return float64(count)
}

func escape(side int, maxIter int) float64 {
	total := 0
	for y := 0; y < side; y++ {
		ci := -1.25 + 2.5*(float64(y)/float64(side))
		for x := 0; x < side; x++ {
			cr := -2.0 + 3.0*(float64(x)/float64(side))
			zr, zi := 0.0, 0.0
			i := 0
			for i < maxIter && zr*zr+zi*zi <= 4.0 {
				t := zr*zr - zi*zi + cr
				zi = 2*zr*zi + ci
				zr = t
				i++
			}
			total += i
		}
	}
	return float64(total)
}

func mix(rounds uint32) float64 {
	var h uint32 = 0x811C9DC5
	for i := uint32(0); i < rounds; i++ {
		h ^= i
		h *= 0x01000193
		h ^= h >> 15
		h += h << 7
	}
	return float64(h)
}

// matmulHinted is byte-for-byte the same arithmetic as matmul, in the same
// order, but sub-slices each row first. That lets the compiler prove the inner
// index is in range and drop the per-element bounds check — the idiomatic Go
// way to write this loop, kept alongside the naive one so the cost of those
// checks can be read straight off the board.
func matmulHinted(n int) float64 {
	a := make([]float64, n*n)
	b := make([]float64, n*n)
	for i := 0; i < n*n; i++ {
		a[i] = float64(i%17)*0.125 - 1.0
		b[i] = float64(i%23)*0.0625 - 0.5
	}

	c := make([]float64, n*n)
	for i := 0; i < n; i++ {
		ci := c[i*n : i*n+n]
		for k := 0; k < n; k++ {
			aik := a[i*n+k]
			if aik == 0 {
				continue
			}
			bk := b[k*n : k*n+n]
			for j := range ci {
				ci[j] += aik * bk[j]
			}
		}
	}

	sum := 0.0
	for _, v := range c {
		sum += v
	}
	return sum
}

// Least-significant-digit radix sort, four 8-bit passes over n words.
func radix(n int) float64 {
	src := make([]uint32, n)
	var x uint32 = 0x92961E37
	for i := range src {
		x ^= x << 13
		x ^= x >> 17
		x ^= x << 5
		src[i] = x
	}

	dst := make([]uint32, n)
	for _, shift := range [4]uint{0, 8, 16, 24} {
		var count [256]uint32
		for _, v := range src {
			count[(v>>shift)&0xFF]++
		}
		var sum uint32
		for i := range count {
			c := count[i]
			count[i] = sum
			sum += c
		}
		for _, v := range src {
			bucket := (v >> shift) & 0xFF
			dst[count[bucket]] = v
			count[bucket]++
		}
		src, dst = dst, src
	}

	var h uint32 = 0x811C9DC5
	for _, v := range src {
		h ^= v
		h *= 0x01000193
	}
	return float64(h)
}

// Table-driven CRC-32 over a generated buffer, folded `rounds` times.
func crc32Bench(length int, rounds int) float64 {
	var table [256]uint32
	for i := range table {
		c := uint32(i)
		for k := 0; k < 8; k++ {
			if c&1 != 0 {
				c = 0xEDB88320 ^ (c >> 1)
			} else {
				c >>= 1
			}
		}
		table[i] = c
	}

	data := make([]byte, length)
	for i := range data {
		data[i] = byte((i * 167) ^ (i >> 3))
	}

	var crc uint32 = 0xFFFFFFFF
	for r := 0; r < rounds; r++ {
		for _, b := range data {
			crc = (crc >> 8) ^ table[(crc^uint32(b))&0xFF]
		}
	}
	return float64(crc ^ 0xFFFFFFFF)
}

// Counts n-queens solutions with a bitmask search.
func queensSearch(cols, ld, rd, all uint32) uint32 {
	if cols == all {
		return 1
	}
	var count uint32
	open := ^(cols | ld | rd) & all
	for open != 0 {
		bit := open & -open
		open -= bit
		count += queensSearch(cols|bit, ((ld|bit)<<1)&all, (rd|bit)>>1, all)
	}
	return count
}

func queens(n uint) float64 {
	all := uint32(1)<<n - 1
	return float64(queensSearch(0, 0, 0, all))
}

type node struct{ left, right *node }

func buildTree(depth int) *node {
	if depth == 0 {
		return &node{}
	}
	return &node{left: buildTree(depth - 1), right: buildTree(depth - 1)}
}

func checkTree(n *node) int64 {
	if n.left == nil {
		return 1
	}
	return 1 + checkTree(n.left) + checkTree(n.right)
}

// Builds and walks many short-lived binary trees — the one task dominated by
// allocation, where a tracing collector shows up most clearly.
func trees(maxDepth int) float64 {
	var total int64
	for depth := 4; depth <= maxDepth; depth += 2 {
		iterations := 1 << uint(maxDepth-depth+4)
		for i := 0; i < iterations; i++ {
			total += checkTree(buildTree(depth))
		}
	}
	return float64(total)
}

func fn(f func(args []js.Value) float64) js.Func {
	return js.FuncOf(func(_ js.Value, args []js.Value) any { return f(args) })
}

func main() {
	js.Global().Set("goBench", map[string]any{
		"matmul":       fn(func(a []js.Value) float64 { return matmul(a[0].Int()) }),
		"matmulHinted": fn(func(a []js.Value) float64 { return matmulHinted(a[0].Int()) }),
		"sieve":        fn(func(a []js.Value) float64 { return sieve(a[0].Int()) }),
		"escape":       fn(func(a []js.Value) float64 { return escape(a[0].Int(), a[1].Int()) }),
		"mix":          fn(func(a []js.Value) float64 { return mix(uint32(a[0].Int())) }),
		"radix":        fn(func(a []js.Value) float64 { return radix(a[0].Int()) }),
		"crc":          fn(func(a []js.Value) float64 { return crc32Bench(a[0].Int(), a[1].Int()) }),
		"queens":       fn(func(a []js.Value) float64 { return queens(uint(a[0].Int())) }),
		"trees":        fn(func(a []js.Value) float64 { return trees(a[0].Int()) }),
	})

	// Tell the page the functions are installed, then stay alive so they
	// remain callable — returning from main would tear the runtime down.
	js.Global().Call("__goBenchReady")
	select {}
}
