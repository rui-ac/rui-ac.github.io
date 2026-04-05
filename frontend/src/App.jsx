import { useState } from 'react'
import styles from './App.module.css'

const STATUS = {
  IDLE: 'idle',
  LOADING: 'loading',
  SUCCESS: 'success',
  ERROR: 'error',
}

export default function App() {
  const [url, setUrl] = useState('')
  const [status, setStatus] = useState(STATUS.IDLE)
  const [errorMsg, setErrorMsg] = useState('')

  async function handleSubmit(e) {
    e.preventDefault()
    const trimmed = url.trim()
    if (!trimmed) return

    setStatus(STATUS.LOADING)
    setErrorMsg('')

    try {
      const res = await fetch('/api/download', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url: trimmed }),
      })

      if (!res.ok) {
        const text = await res.text()
        throw new Error(text || `Server error ${res.status}`)
      }

      const disposition = res.headers.get('Content-Disposition')
      const match = disposition?.match(/filename="?([^"]+)"?/)
      const filename = match?.[1] ?? 'download'

      const blob = await res.blob()
      const objectUrl = URL.createObjectURL(blob)
      const anchor = document.createElement('a')
      anchor.href = objectUrl
      anchor.download = filename
      anchor.click()
      URL.revokeObjectURL(objectUrl)

      setStatus(STATUS.SUCCESS)
      setUrl('')
    } catch (err) {
      setErrorMsg(err.message)
      setStatus(STATUS.ERROR)
    }
  }

  const isLoading = status === STATUS.LOADING

  return (
    <main className={styles.main}>
      <div className={styles.card}>
        <header className={styles.header}>
          <div className={styles.icon}>↓</div>
          <h1 className={styles.title}>Downloader</h1>
          <p className={styles.subtitle}>Paste any gallery or post URL</p>
        </header>

        <form onSubmit={handleSubmit} className={styles.form}>
          <div className={styles.inputWrapper}>
            <input
              type="url"
              className={styles.input}
              placeholder="https://..."
              value={url}
              onChange={(e) => {
                setUrl(e.target.value)
                if (status !== STATUS.IDLE) setStatus(STATUS.IDLE)
              }}
              disabled={isLoading}
              autoComplete="off"
              autoCorrect="off"
              autoCapitalize="off"
              spellCheck="false"
              inputMode="url"
            />
          </div>

          <button
            type="submit"
            className={styles.button}
            disabled={isLoading || !url.trim()}
          >
            {isLoading ? (
              <span className={styles.spinner} aria-label="Downloading" />
            ) : (
              'Download'
            )}
          </button>
        </form>

        {status === STATUS.SUCCESS && (
          <p className={styles.success}>Download started!</p>
        )}
        {status === STATUS.ERROR && (
          <p className={styles.error}>{errorMsg}</p>
        )}
      </div>
    </main>
  )
}
