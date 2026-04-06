---
name: frontend-patterns
description: Frontend development patterns for React, Next.js, state management, performance optimization, and UI best practices.
origin: ECC
---

# Frontend Development Patterns

## When to Activate

- Building React components, managing state, fetching data
- Optimizing performance (memoization, virtualization, code splitting)
- Working with forms, accessibility, or animations

## Component Patterns

### Composition

```typescript
export function Card({ children, variant = 'default' }: CardProps) {
  return <div className={`card card-${variant}`}>{children}</div>
}
export function CardHeader({ children }: { children: React.ReactNode }) {
  return <div className="card-header">{children}</div>
}
export function CardBody({ children }: { children: React.ReactNode }) {
  return <div className="card-body">{children}</div>
}
```

### Compound Components

```typescript
const TabsContext = createContext<TabsContextValue | undefined>(undefined)

export function Tabs({ children, defaultTab }: { children: React.ReactNode; defaultTab: string }) {
  const [activeTab, setActiveTab] = useState(defaultTab)
  return <TabsContext.Provider value={{ activeTab, setActiveTab }}>{children}</TabsContext.Provider>
}

export function Tab({ id, children }: { id: string; children: React.ReactNode }) {
  const context = useContext(TabsContext)
  if (!context) throw new Error('Tab must be used within Tabs')
  return <button className={context.activeTab === id ? 'active' : ''} onClick={() => context.setActiveTab(id)}>{children}</button>
}
```

### Render Props

```typescript
export function DataLoader<T>({ url, children }: DataLoaderProps<T>) {
  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    fetch(url).then(res => res.json()).then(setData).catch(setError).finally(() => setLoading(false))
  }, [url])

  return <>{children(data, loading, error)}</>
}
```

## Custom Hooks

```typescript
export function useToggle(initialValue = false): [boolean, () => void] {
  const [value, setValue] = useState(initialValue)
  const toggle = useCallback(() => setValue(v => !v), [])
  return [value, toggle]
}

export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value)
  useEffect(() => {
    const handler = setTimeout(() => setDebouncedValue(value), delay)
    return () => clearTimeout(handler)
  }, [value, delay])
  return debouncedValue
}
```

## State Management: Context + Reducer

```typescript
type Action = { type: 'SET_MARKETS'; payload: Market[] } | { type: 'SELECT_MARKET'; payload: Market } | { type: 'SET_LOADING'; payload: boolean }

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'SET_MARKETS': return { ...state, markets: action.payload }
    case 'SELECT_MARKET': return { ...state, selectedMarket: action.payload }
    case 'SET_LOADING': return { ...state, loading: action.payload }
    default: return state
  }
}

export function MarketProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(reducer, { markets: [], selectedMarket: null, loading: false })
  return <MarketContext.Provider value={{ state, dispatch }}>{children}</MarketContext.Provider>
}
```

## Performance

### Memoization

```typescript
const sortedMarkets = useMemo(() => markets.sort((a, b) => b.volume - a.volume), [markets])
const handleSearch = useCallback((query: string) => setSearchQuery(query), [])
export const MarketCard = React.memo<MarketCardProps>(({ market }) => (
  <div className="market-card"><h3>{market.name}</h3></div>
))
```

### Code Splitting

```typescript
const HeavyChart = lazy(() => import('./HeavyChart'))
<Suspense fallback={<ChartSkeleton />}><HeavyChart data={data} /></Suspense>
```

### Virtualization

```typescript
import { useVirtualizer } from '@tanstack/react-virtual'

export function VirtualMarketList({ markets }: { markets: Market[] }) {
  const parentRef = useRef<HTMLDivElement>(null)
  const virtualizer = useVirtualizer({
    count: markets.length, getScrollElement: () => parentRef.current,
    estimateSize: () => 100, overscan: 5
  })
  return (
    <div ref={parentRef} style={{ height: '600px', overflow: 'auto' }}>
      <div style={{ height: `${virtualizer.getTotalSize()}px`, position: 'relative' }}>
        {virtualizer.getVirtualItems().map(virtualRow => (
          <div key={virtualRow.index} style={{ position: 'absolute', top: 0, width: '100%', height: `${virtualRow.size}px`, transform: `translateY(${virtualRow.start}px)` }}>
            <MarketCard market={markets[virtualRow.index]} />
          </div>
        ))}
      </div>
    </div>
  )
}
```

## Form Handling

```typescript
export function CreateMarketForm() {
  const [formData, setFormData] = useState<FormData>({ name: '', description: '', endDate: '' })
  const [errors, setErrors] = useState<FormErrors>({})

  const validate = (): boolean => {
    const newErrors: FormErrors = {}
    if (!formData.name.trim()) newErrors.name = 'Name is required'
    if (!formData.description.trim()) newErrors.description = 'Description is required'
    if (!formData.endDate) newErrors.endDate = 'End date is required'
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!validate()) return
    await createMarket(formData)
  }

  return (
    <form onSubmit={handleSubmit}>
      <input value={formData.name} onChange={e => setFormData(prev => ({ ...prev, name: e.target.value }))} />
      {errors.name && <span className="error">{errors.name}</span>}
      <button type="submit">Create</button>
    </form>
  )
}
```

## Error Boundary

```typescript
export class ErrorBoundary extends React.Component<{ children: React.ReactNode }, { hasError: boolean; error: Error | null }> {
  state = { hasError: false, error: null as Error | null }
  static getDerivedStateFromError(error: Error) { return { hasError: true, error } }
  componentDidCatch(error: Error, info: React.ErrorInfo) { console.error('Error boundary:', error, info) }
  render() {
    if (this.state.hasError) return (
      <div className="error-fallback">
        <h2>Something went wrong</h2>
        <p>{this.state.error?.message}</p>
        <button onClick={() => this.setState({ hasError: false })}>Try again</button>
      </div>
    )
    return this.props.children
  }
}
```

## Accessibility

```typescript
// Keyboard navigation
const handleKeyDown = (e: React.KeyboardEvent) => {
  switch (e.key) {
    case 'ArrowDown': e.preventDefault(); setActiveIndex(i => Math.min(i + 1, options.length - 1)); break
    case 'ArrowUp': e.preventDefault(); setActiveIndex(i => Math.max(i - 1, 0)); break
    case 'Enter': e.preventDefault(); onSelect(options[activeIndex]); break
    case 'Escape': setIsOpen(false); break
  }
}

// Focus management for modals
useEffect(() => {
  if (isOpen) { previousFocusRef.current = document.activeElement as HTMLElement; modalRef.current?.focus() }
  else { previousFocusRef.current?.focus() }
}, [isOpen])
```
