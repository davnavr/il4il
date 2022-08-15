namespace Il4ilSharp;

using System;
using Il4ilSharp.Interop;

/// <summary>Indicates the name of an IL4IL module in a metadata section.</summary>
public sealed class ModuleNameMetadata : ModuleMetadata {
    /// <summary>Gets the module name.</summary>
    public IdentifierString Name { get; }

    internal ModuleNameMetadata(MetadataHandle handle) : base(handle, Interop.Native.MetadataKind.ModuleName) {
        Name = new IdentifierString(handle.GetModuleName() ?? throw new InvalidOperationException("not a name"));
    }

    /// <summary>Returns a <see cref="String"/> containing the module name.</summary>
    public override string ToString() => Name.ToString();
}
