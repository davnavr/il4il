namespace Il4ilSharp.Interop;

using System;
using System.Threading;

/// <summary>
/// Base class that provides thread-safe access to an underlying resource, only disposing the resource on finalization.
/// </summary>
/// <remarks>
/// <p>
/// This class does not implement <see cref="IDisposable"/>, as it is intended to allow shared ownership of an underlying resource.
/// </p>
/// <p>
/// For single ownership and explicit disposable of the underlying resource, use the <see cref="SynchronizedHandle{T}"/> class instead.
/// </p>
/// </remarks>
public unsafe abstract class SharedHandle<T> where T : unmanaged {
    private readonly object sync = new();

    private T* pointer;

    /// <summary>Initializes a <see cref="SharedHandle{T}"/> for the specified <paramref name="pointer"/>.</summary>
    /// <exception cref="ArgumentNullException">Thrown when the <paramref name="pointer"/> is <see langword="null"/>.</exception>
    internal SharedHandle(T* pointer) {
        if (pointer == null) {
            throw new ArgumentNullException(nameof(pointer));
        }

        this.pointer = pointer;
    }

    /// <summary>Acquires an exclusive lock for the handle, ensuring only the current thread has access to the underlying pointer.</summary>
    /// <remarks>To release the lock, callers of this method must then call <see cref="Unlock"/>.</remarks>
    public T* Lock() {
        Monitor.Enter(sync);
        return pointer;
    }

    /// <summary>Releases an exclusive lock for the handle.</summary>
    /// <remarks>
    /// Callers should ensure that the pointer returned by a previous call to <see cref="Lock"/> is no longer used after this method is called.
    /// </remarks>
    internal void Unlock() => Monitor.Exit(sync);

    /// <summary>Disposes the underlying resource.</summary>
    private protected abstract void Cleanup(T* pointer);
}
