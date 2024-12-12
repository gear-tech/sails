namespace Substrate.Gear.Client.Extensions;

public static class ReadOnlyMemoryExtensions
{
    /// <summary>
    /// Returns a read-only collection that wraps the specified memory.
    /// </summary>
    /// <typeparam name="T"></typeparam>
    /// <param name="memory"></param>
    /// <returns></returns>
    public static IReadOnlyCollection<T> AsReadOnlyCollection<T>(this ReadOnlyMemory<T> memory)
        => new ReadOnlyMemoryCollection<T>(memory);

    private readonly struct ReadOnlyMemoryCollection<T> : IReadOnlyCollection<T>
    {
        public ReadOnlyMemoryCollection(ReadOnlyMemory<T> memory)
        {
            this.memory = memory;
        }

        private readonly ReadOnlyMemory<T> memory;

        public int Count => this.memory.Length;

        public IEnumerator<T> GetEnumerator()
        {
            for (var i = 0; i < this.memory.Length; i++)
            {
                yield return this.memory.Span[i];
            }
        }

        IEnumerator IEnumerable.GetEnumerator() => this.GetEnumerator();
    }
}
