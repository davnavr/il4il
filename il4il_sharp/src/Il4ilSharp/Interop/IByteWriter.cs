namespace Il4ilSharp.Interop;

/// <summary>Interface for destinations that bytes can be written to.</summary>
public interface IByteWriter {
    /// <summary>Writes the given bytes.</summary>
    void Write(System.ReadOnlySpan<byte> bytes);
}
