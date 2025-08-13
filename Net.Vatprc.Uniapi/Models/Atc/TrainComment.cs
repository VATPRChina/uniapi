namespace Net.Vatprc.Uniapi.Models.Atc;

public class TrainComment
{
    public long Id { get; set; }

    public decimal Train { get; set; }

    public decimal Author { get; set; }

    public decimal LastEditor { get; set; }

    public string Content { get; set; } = null!;

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public virtual User AuthorNavigation { get; set; } = null!;

    public virtual User LastEditorNavigation { get; set; } = null!;
}
