namespace Il4ilSharp.Interop;

using System;
using System.IO;

/// <summary>Encapsulates a <see cref="System.IO.Stream"/> as a destination that bytes can be written to.</summary>
public sealed class StreamByteWriter<S> : IDisposable, IByteWriter where S : Stream {
    /// <summary>Gets the underlying <see cref="System.IO.Stream"/> that bytes can be written to.</summary>
    public S Stream { get; }

    /// <summary>Initializes a new <see cref="StreamByteWriter{S}"/> with an underlying <see cref="System.IO.Stream"/>.</summary>
    /// <exception cref="ArgumentNullException">Thrown if the <paramref name="stream"/> is <see langword="null"/>.</exception>
    public StreamByteWriter(S stream) {
        ArgumentNullException.ThrowIfNull(stream);

        if (!stream.CanWrite) {
            throw new ArgumentException(nameof(stream), "Destination stream must support writing");
        }

        Stream = stream;
    }

    /// <inheritdoc/>
    public void Write(ReadOnlySpan<byte> bytes) => Stream.Write(bytes);

    /// <inheritdoc/>
    public void Dispose() => Stream.Dispose();
}
