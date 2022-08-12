namespace Il4ilSharp.Interop;

using System.Runtime.InteropServices;

/// <summary>Methods for manipulating IL4IL identifier strings.</summary>
public unsafe static class Identifier
{
    public readonly ref struct Opaque { }

    [DllImport(Native.LibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_from_utf8", ExactSpelling = true)]
    public static extern Opaque* FromUtf8(byte* contents, nuint length, Error.Opaque* error);

    [DllImport(Native.LibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_contents", ExactSpelling = true)]
    public static extern byte* Contents(Opaque* identifier, nuint* length, Error.Opaque* error);

    [DllImport(Native.LibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_dispose", ExactSpelling = true)]
    public static extern void Dispose(Opaque* identifier, Error.Opaque* error);
}
