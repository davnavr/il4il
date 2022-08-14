namespace Il4ilSharp.Interop.Native;

using System.Runtime.InteropServices;

/// <summary>Methods for manipulating IL4IL identifier strings.</summary>
public unsafe static class Identifier {
    public readonly struct Opaque { }

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_from_utf8", ExactSpelling = true)]
    public static extern Error.Opaque* FromUtf8(byte* contents, nuint length, out Opaque* identifier);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_from_utf16", ExactSpelling = true)]
    public static extern Error.Opaque* FromUtf16(char* contents, nuint count, out Opaque* identifier);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_byte_length", ExactSpelling = true)]
    public static extern nuint ByteLength(Opaque* identifier);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_copy_bytes_to", ExactSpelling = true)]
    public static extern nuint CopyBytesTo(Opaque* identifier, byte* buffer);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_dispose", ExactSpelling = true)]
    public static extern void Dispose(Opaque* identifier);
}
