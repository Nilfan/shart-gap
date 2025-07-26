import React, { useState, useEffect, useRef } from 'react'
import { Party, User, Message } from '../../App'
import styles from './ChatArea.module.css'

interface ChatAreaProps {
  room: Party
  currentUser: User | null
  onSendMessage: (content: string) => void
}

function ChatArea({ room, currentUser, onSendMessage }: ChatAreaProps) {
  const [messages, setMessages] = useState<Message[]>([])
  const [messageInput, setMessageInput] = useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    // Load messages for the room from room data
    console.log('🔄 ChatArea: Loading messages for room:', room.name, 'ID:', room.id)
    setMessages(room.messages || [])
  }, [room.id, room.messages])

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  const handleSendMessage = (e: React.FormEvent) => {
    e.preventDefault()
    if (messageInput.trim() && currentUser) {
      onSendMessage(messageInput.trim())
      setMessageInput('')
    }
  }

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const isToday = date.toDateString() === now.toDateString()
    
    if (isToday) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    } else {
      return date.toLocaleDateString([], { 
        month: 'short', 
        day: 'numeric',
        hour: '2-digit', 
        minute: '2-digit' 
      })
    }
  }

  return (
    <div className={styles.chatArea}>
      <div className={styles.header}>
        <div className={styles.roomInfo}>
          <h2 className={styles.roomName}>{room.name}</h2>
          <div className={styles.roomMeta}>
            <span className={styles.userCount}>{Object.keys(room.users).length} members</span>
            <div className={styles.protocolBadge}>
              Protocol: {room.protocol}
            </div>
          </div>
        </div>
        
        <div className={styles.headerActions}>
          <button 
            className={`${styles.actionButton} ${room.is_voice_enabled ? styles.active : ''}`}
            title={room.is_voice_enabled ? 'Leave Call' : 'Join Call'}
          >
            {room.is_voice_enabled ? '🔇' : '🎤'}
          </button>
          <button className={styles.actionButton} title="Room Settings">
            ⚙️
          </button>
        </div>
      </div>

      <div className={styles.messagesContainer}>
        <div className={styles.messagesList}>
          {messages.length === 0 ? (
            <div className={styles.emptyState}>
              <h3>Welcome to #{room.name}</h3>
              <p>This is the beginning of your conversation in this room.</p>
            </div>
          ) : (
            messages.map((message) => (
              <div key={message.id} className={`${styles.message} ${message.userName === 'System' ? styles.systemMessage : ''}`}>
                {message.userName === 'System' ? (
                  <div className={styles.systemContent}>
                    {message.content}
                  </div>
                ) : (
                  <>
                    <div className={styles.messageHeader}>
                      <span className={styles.userName}>{message.userName}</span>
                      <span className={styles.timestamp}>
                        {formatTime(message.timestamp)}
                      </span>
                    </div>
                    <div className={styles.messageContent}>
                      {message.content}
                    </div>
                  </>
                )}
              </div>
            ))
          )}
          <div ref={messagesEndRef} />
        </div>
      </div>

      <div className={styles.messageInput}>
        <form onSubmit={handleSendMessage} className={styles.inputForm}>
          <input
            type="text"
            placeholder={`Message #${room.name}`}
            value={messageInput}
            onChange={(e) => setMessageInput(e.target.value)}
            className={styles.textInput}
          />
          <button 
            type="submit" 
            className={styles.sendButton}
            disabled={!messageInput.trim()}
          >
            Send
          </button>
        </form>
      </div>
    </div>
  )
}

export default ChatArea