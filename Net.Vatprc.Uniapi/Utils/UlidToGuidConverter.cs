using Microsoft.EntityFrameworkCore.Storage.ValueConversion;

namespace Net.Vatprc.Uniapi.Utils;

public class UlidToGuidConverter(ConverterMappingHints? mappingHints = null) : ValueConverter<Ulid, Guid>(
            convertToProviderExpression: x => x.ToGuid(),
            convertFromProviderExpression: x => new Ulid(x),
            mappingHints: defaultHints.With(mappingHints))
{
    private static readonly ConverterMappingHints defaultHints = new(size: 16);

    public UlidToGuidConverter() : this(null)
    {
    }
}
