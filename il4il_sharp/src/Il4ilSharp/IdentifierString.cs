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

    public IdentifierString(IdentifierHandle handle) {
        ArgumentNullException.ThrowIfNull(handle);

        if (handle.IsDisposed) {
            throw new ArgumentException(nameof(handle), "Handle was already disposed");
        }

        Handle = handle;
    }
}
