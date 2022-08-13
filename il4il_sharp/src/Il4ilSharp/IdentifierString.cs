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
    /// <exception cref="ArgumentException">Thrown when the <paramref name="handle"/> was already disposed.</exception>
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

    /// <summary>Initializes a <see cref="IdentifierString"/> with the given UTF-16 code points.</summary>
    /// <exception cref="ArgumentException">
    /// Thrown when the <paramref name="contents"/> are empty or contain <c>NUL</c> bytes.
    /// </exception>
    public IdentifierString(ReadOnlySpan<char> contents) {
        Handle = new IdentifierHandle(contents);
        cached = new string(contents);
    }

    /// <summary>Initializes a <see cref="IdentifierString"/> from a UTF-16 string.</summary>
    /// <exception cref="ArgumentNullException">
    /// Thrown when the <paramref name="contents"/> is <see langword="null"/>.
    /// </exception>
    /// <exception cref="ArgumentException">
    /// Thrown when the <paramref name="contents"/> are empty or contains the null character <c>'\0'.</c>.
    /// </exception>
    public IdentifierString(string contents) {
        Handle = new IdentifierHandle(contents);
        cached = contents;
    }

    /// <summary>Returns the contents of the identifier string.</summary>
    public override string ToString() => cached;
}
