namespace Il4ilSharp.Interop;

using System;
using System.Buffers;
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

    private static Identifier.Opaque* Allocate(ReadOnlySpan<char> characters) {
        fixed (char* contents = characters) {
            Identifier.Opaque* identifier;

            try {
                ErrorHandling.Throw(Identifier.FromUtf16(contents, (nuint)characters.Length, out identifier));
            } catch (ErrorHandlingException e) {
                throw new ArgumentException(nameof(characters), e);
            }

            return identifier;
        }
    }

    /// <summary>
    /// Initializes a new <see cref="IdentifierHandle"/>, allocating a new identifier string from a sequence of UTF-16 codepoints.
    /// </summary>
    /// <exception cref="ArgumentException">
    /// Thrown when the <paramref name="contents"/> are empty or contain <c>NUL</c> bytes.
    /// </exception>
    public IdentifierHandle(ReadOnlySpan<char> contents) : this(Allocate(contents)) { }

    /// <summary>
    /// Initializes a new <see cref="IdentifierHandle"/>, copying the contents of the specified <see cref="String"/>.
    /// </summary>
    public IdentifierHandle(string contents) : this((ReadOnlySpan<char>)(contents ?? throw new ArgumentNullException(nameof(contents)))) { }

    /// <summary>Attempts to copy the UTF-8 contents of the identifier string into a newly allocated byte array.</summary>
    /// <returns>An array containing the UTF-8 string, or an empty array if the identifier was disposed.</returns>
    public byte[] ToArray() {
        if (IsDisposed) {
            return Array.Empty<byte>();
        }

        try {
            var identifier = Enter();
            byte[] buffer = new byte[(int)Identifier.ByteLength(identifier)];
            fixed (byte* bytes = buffer) {
                Identifier.CopyBytesTo(identifier, bytes);
            }

            return buffer;
        } finally {
            Exit();
        }
    }

    /// <summary>Attempts to convert the identifier string to a .NET <see cref="String"/>.</summary>
    /// <returns>The contents of the identifier string, or <see cref="String.Empty"/> if the identifier was disposed.</returns>
    public override string ToString() {
        if (IsDisposed) {
            return String.Empty;
        }

        byte[]? rented = null;

        try {
            var identifier = Enter();
            int length = (int)Identifier.ByteLength(identifier);
            Span<byte> buffer = length > 256 ? stackalloc byte[length] : rented = ArrayPool<byte>.Shared.Rent(length);
            fixed (byte* bytes = buffer) {
                Identifier.CopyBytesTo(identifier, bytes);
            }

            return Encoding.UTF8.GetString(buffer);
        } finally {
            Exit();

            if (rented != null) {
                ArrayPool<byte>.Shared.Return(rented);
            }
        }
    }

    private protected override unsafe void Cleanup(Identifier.Opaque* pointer) => Identifier.Dispose(pointer);
}
