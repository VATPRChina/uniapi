using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;

namespace Net.Vatprc.Uniapi.Adapters;

[TestFixture]
public class VatsimAuthAdapterTest
{
    static string ToBase64Url(byte[] data) =>
        Convert.ToBase64String(data).Replace('+', '-').Replace('/', '_').TrimEnd('=');

    [Test]
    public void GeneratePkce_Default_ReturnsNonEmptyUrlSafeValues_NoPadding()
    {
        var (challenge, verifier) = VatsimAuthAdapter.GeneratePkce();

        string.IsNullOrWhiteSpace(verifier).Should().BeFalse();
        string.IsNullOrWhiteSpace(challenge).Should().BeFalse();
        verifier.Contains('=').Should().BeFalse();
        challenge.Contains('=').Should().BeFalse();

        var urlSafeRegex = new Regex("^[A-Za-z0-9\\-_]+$");
        urlSafeRegex.IsMatch(verifier).Should().BeTrue();
        urlSafeRegex.IsMatch(challenge).Should().BeTrue();
    }

    [Test]
    public void GeneratePkce_ChallengeIsSha256OfVerifier()
    {
        var (challenge, verifier) = VatsimAuthAdapter.GeneratePkce();

        var buffer = Encoding.UTF8.GetBytes(verifier);
        var hash = SHA256.HashData(buffer);
        var expected = ToBase64Url(hash);

        challenge.Should().Be(expected);
    }
}
