namespace Il4ilSharp.Interop;

using System;

/// <summary>Base class that provides thread-safe access to a pointer derived from a <see cref="SharedHandle{T}"/>.</summary>
public unsafe abstract class DerivedHandle<P, O, C>
    where P : SharedHandle<O>
    where O : unmanaged
    where C : unmanaged {
    private readonly C* derived;

    /// <summary>
    /// Gets a <see cref="SharedHandle{O}"/> encapsulating the pointer to the data that the derived handle contains a pointer to.
    /// </summary>
    public P Parent { get; }

    internal DerivedHandle(P parent, C* derived) {
        ArgumentNullException.ThrowIfNull(parent);
        if (derived == null) {
            throw new ArgumentNullException(nameof(derived));
        }

        Parent = parent;
        this.derived = derived;
    }

    internal C* Lock() {
        Parent.Lock();
        return derived;
    }

    internal void Unlock() => Parent.Unlock();
}
