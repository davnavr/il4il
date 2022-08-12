namespace Il4ilSharp.Interop;

using System;
using System.Text;
using Il4ilSharp.Interop.Native;

/// <summary>
/// Provides a thread-safe wrapper for an IL4IL identifier string.
/// </summary>
/// <seealso cref="IdentifierString"/>
public unsafe sealed class IdentifierHandle : SyncHandle<Identifier.Opaque> {
    /// <summary>Gets a value indicating whether the underlying identifier string was disposed.</summary>
    public new bool IsDisposed => base.IsDisposed;

    internal IdentifierHandle(Identifier.Opaque* identifier) : base(identifier) { }

    private static ReadOnlySpan<byte> Contents(Identifier.Opaque* identifier) {
        nuint length;
        Error.Opaque* error;
        byte* contents = Identifier.Contents(identifier, out length, out error);
        ErrorHandling.Throw(error);
        return new ReadOnlySpan<byte>(contents, (int)length);
    }

    /// <summary>Attempts to convert the identifier string to a .NET <see cref="String"/>.</summary>
    /// <returns>The contents of the identifier string, or <see cref="String.Empty"/> if the identifier was disposed.</returns>
    public override string ToString() {
        if (IsDisposed) {
            return String.Empty;
        }

        try {
            var identifier = Enter();
            return Encoding.UTF8.GetString(Contents(identifier));
        } finally {
            Exit();
        }
    }

    /// <summary>Attempts to copy the UTF-8 contents of the identifier string into a newly allocated byte array.</summary>
    /// <returns>An array containing the UTF-8 string, or an empty array if the identifier was disposed.</returns>
    public byte[] ToArray() {
        if (IsDisposed) {
            return Array.Empty<byte>();
        }

        try {
            var identifier = Enter();
            return Contents(identifier).ToArray();
        } finally {
            Exit();
        }
    }

    private protected override unsafe void Cleanup(Identifier.Opaque* pointer) {
        Error.Opaque* error;
        Identifier.Dispose(pointer, out error);
        ErrorHandling.Throw(error);
    }
}
