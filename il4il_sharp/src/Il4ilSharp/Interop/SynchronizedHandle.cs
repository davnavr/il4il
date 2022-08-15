namespace Il4ilSharp.Interop;

using System;
using System.Threading;

/// <summary>Base class that provides thread-safe, mutually exclusive access to an underlying resource.</summary>
/// <remarks>This class implements <see cref="IDisposable"/> to allow explicit disposal of the underlying resource.</remarks>
public unsafe abstract class SynchronizedHandle<T> : IDisposable where T : unmanaged {
    private readonly object sync = new();

    private T* pointer;

    private protected SynchronizedHandle(T* pointer) {
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
    /// <remarks>To release the lock, callers of this method must then call <see cref="Unlock"/>.</remarks>
    /// <exception cref="ObjectDisposedException">Thrown if the handle was already disposed.</exception>
    internal T* Lock() {
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
        T* pointer = Lock();
        this.pointer = null;
        Unlock();
        GC.SuppressFinalize(this);
        return pointer;
    }

    /// <summary>Releases an exclusive lock for the handle.</summary>
    /// <remarks>
    /// Callers should ensure that the pointer returned by a previous call to <see cref="Lock"/> is no longer used after this method is called.
    /// </remarks>
    internal void Unlock() => Monitor.Exit(sync);

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
                Unlock();
            }
        }
    }

    /// <inheritdoc/>
    public void Dispose() {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    /// <inheritdoc/>
    ~SynchronizedHandle() => Dispose(false);
}
