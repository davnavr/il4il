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

    /// <summary>
    /// Acquires an exclusive lock for the handle, ensuring only the current thread has access to the underlying pointer.
    /// </summary>
    /// <remarks>To release the lock, callers of this method must then call <see cref="Exit"/>.</remarks>
    /// <exception cref="ObjectDisposedException">Thrown if the handle was already disposed.</exception>
    internal T* Enter() {
        Monitor.Enter(sync);

        if (IsDisposed) {
            throw new ObjectDisposedException(GetType().FullName);
        }

        return pointer;
    }

    /// <summary>Takes ownership of the underlying pointer, leaving this handle unusable.</summary>
    /// <remarks>Callers are responsible for disposing the underlying resource and ensuring thread safety.</remarks>
    /// <exception cref="ObjectDisposedException">Thrown if the handle was already disposed.</exception>
    internal T* Take() {
        T* pointer = Enter();
        this.pointer = null;
        Exit();
        GC.SuppressFinalize(this);
        return pointer;
    }

    /// <summary>
    /// <p>Releases an exclusive lock for the handle.</p>
    /// <p>Callers should ensure that the underlying pointer is not used until another call to <see cref="Enter"/>.</p>
    /// </summary>
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
            if (disposing) {
                Exit();
            }
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
