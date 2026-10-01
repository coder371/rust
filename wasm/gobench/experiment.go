//go:build js && wasm && probe

// Diagnostic probes, excluded from the shipped module. Build with
//
//	GOOS=js GOARCH=wasm go build -tags probe -o ../pkg/go_bench.wasm .
//
// Registered separately from the benchmark itself. They
// exist to attribute Go's gap to a cause rather than assert one: each isolates
// a single suspected source of overhead while leaving everything else alone.
package main

import (
	"runtime"
	"runtime/debug"
	"syscall/js"
)

// treesArena builds the same trees, but hands out nodes from one big slab
// instead of calling the allocator per node. Everything else — the recursion,
// the pointer writes, the traversal — is identical, so the difference is the
// cost of `new` itself.
type arena struct {
	slab []node
	next int
}

func (a *arena) get() *node {
	if a.next == len(a.slab) {
		a.slab = make([]node, len(a.slab))
		a.next = 0
	}
	n := &a.slab[a.next]
	a.next++
	n.left, n.right = nil, nil
	return n
}

func buildArena(a *arena, depth int) *node {
	n := a.get()
	if depth > 0 {
		n.left = buildArena(a, depth-1)
		n.right = buildArena(a, depth-1)
	}
	return n
}

func treesArena(maxDepth int) float64 {
	var total int64
	a := &arena{slab: make([]node, 1<<20)}
	for depth := 4; depth <= maxDepth; depth += 2 {
		iterations := 1 << uint(maxDepth-depth+4)
		for i := 0; i < iterations; i++ {
			a.next = 0
			total += checkTree(buildArena(a, depth))
		}
	}
	return float64(total)
}

func init() {
	js.Global().Set("goExperiment", map[string]any{
		"treesArena": fn(func(a []js.Value) float64 { return treesArena(a[0].Int()) }),

		// Turn the collector off around a timed call, so the very same code
		// can be measured with and without reclamation.
		"gcOff": js.FuncOf(func(js.Value, []js.Value) any {
			debug.SetGCPercent(-1)
			return nil
		}),
		"gcOn": js.FuncOf(func(js.Value, []js.Value) any {
			debug.SetGCPercent(100)
			runtime.GC()
			return nil
		}),
		"heapMB": js.FuncOf(func(js.Value, []js.Value) any {
			var m runtime.MemStats
			runtime.ReadMemStats(&m)
			return float64(m.HeapAlloc) / 1048576.0
		}),
	})
}
