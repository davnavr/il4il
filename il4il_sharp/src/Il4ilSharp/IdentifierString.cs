namespace Il4ilSharp;

using System;
using Il4ilSharp.Interop;

/// <summary>
/// Represents an IL4IL identifier, which is a UTF-8 encoded string that cannot be empty or contain <see langword="null"/> bytes.
/// </summary>
public sealed class IdentifierString {
    private readonly string cached;

    /// <summary>Gets a wrapper for the underlying identifier string.</summary>
    public IdentifierHandle Handle { get; }

    /// <summary>Initializes a <see cref="IdentifierString"/> with the given <paramref name="handle"/>.</summary>
    /// <exception cref="ArgumentNullException">Thrown when the <paramref name="handle"/> is <see langword="null"/>.</exception>
    /// <exception cref="ArgumentException">Throw when the <paramref name="handle"/> was already disposed.</exception>
    public IdentifierString(IdentifierHandle handle) {
        ArgumentNullException.ThrowIfNull(handle);

        try {
            unsafe {
                handle.Enter(); // Prevent handle from being disposed early.
            }

            if (handle.IsDisposed) {
                throw new ArgumentException(nameof(handle), "Handle was already disposed");
            }

            cached = handle.ToString();
            Handle = handle;
        } finally {
            handle.Exit();
        }
    }

    /// <summary>Returns the contents of the identifier string.</summary>
    public override string ToString() => cached;
}
