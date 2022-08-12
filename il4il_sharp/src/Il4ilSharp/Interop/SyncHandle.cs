namespace Il4ilSharp.Interop;

using System;
using System.Threading;

/// <summary>Base class that provides thread-safe access to an underlying native pointer.</summary>
public unsafe abstract class SyncHandle<T> : IDisposable where T : unmanaged {
    private readonly object sync = new();

    private T* pointer;

    private protected SyncHandle(T* pointer) {
        if (pointer == null) {
            throw new ArgumentNullException(nameof(pointer));
        }

        this.pointer = pointer;
    }

    /// <summary>Indicates whether the underlying object was already disposed.</summary>
    public bool IsDisposed => pointer == null;

    internal T* Enter() {
        Monitor.Enter(sync);
        return pointer;
    }

    internal void Exit() => Monitor.Exit(sync);

    private protected abstract void Cleanup(T* pointer);

    private void Dispose(bool disposing) {
        try {
            if (disposing) {
                Monitor.Enter(sync);
            }

            if (!IsDisposed) {
                T* t = pointer;
                pointer = null; // Mark the handle as disposed, even if an exception occurs during cleanup.
                Cleanup(t);
            }
        } finally {
            Exit();
        }
    }

    /// <inheritdoc/>
    public void Dispose() {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    /// <inheritdoc/>
    ~SyncHandle() => Dispose(false);
}
