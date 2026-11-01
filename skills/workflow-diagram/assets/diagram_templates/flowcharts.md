## Process Flow Templates

### Linear Development Process
```mermaid
graph TD
    A[Idea] --> B[Requirements]
    B --> C[Design]
    C --> D[Implementation]
    D --> E[Testing]
    E --> F[Deployment]
    F --> G[Monitoring]
    
    style A fill:#e3f2fd
    style B fill:#e3f2fd
    style C fill:#e8f5e9
    style D fill:#e8f5e9
    style E fill:#fff3e0
    style F fill:#ffebee
    style G fill:#4caf50
```

### Iterative Process with Feedback
```mermaid
graph LR
    A[Plan] --> B[Execute]
    B --> C[Review]
    C --> D{Good?}
    D -->|Yes| E[Complete]
    D -->|No| F[Learn]
    F --> A
    
    style C fill:#ff9800
    style D fill:#ff9800
    style E fill:#4caf50
```

### Decision Tree Process
```mermaid
graph TD
    A[Start] --> B{Type A?}
    B -->|Yes| C[Process A]
    B -->|No| D{Type B?}
    D -->|Yes| E[Process B]
    D -->|No| F[Process C]
    C --> G[End]
    E --> G
    F --> G
    
    style B fill:#ff9800
    style D fill:#ff9800
    style G fill:#4caf50
```

## Task Management Templates

### Task Breakdown Structure
```mermaid
graph TD
    A[Main Task] --> B[Sub-task 1]
    A --> C[Sub-task 2]
    A --> D[Sub-task 3]
    B --> E[Action 1.1]
    B --> F[Action 1.2]
    C --> G[Action 2.1]
    C --> H[Action 2.2]
    D --> I[Action 3.1]
    
    E --> J[Integration]
    F --> J
    G --> J
    H --> J
    I --> J
    
    style A fill:#2196f3
    style J fill:#4caf50
```

### Parallel Task Execution
```mermaid
graph TD
    A[Start] --> B[Task 1]
    A --> C[Task 2]
    A --> D[Task 3]
    B --> E[Review 1]
    C --> F[Review 2]
    D --> G[Review 3]
    E --> H[Final Integration]
    F --> H
    G --> H
    
    style H fill:#4caf50
```

## Review Process Templates

### Code Review Workflow
```mermaid
graph TD
    A[Submit PR] --> B[Automated Checks]
    B --> C{Tests Pass?}
    C -->|No| D[Fix Issues]
    C -->|Yes| E[Peer Review]
    E --> F{Approved?}
    F -->|No| G[Address Feedback]
    F -->|Yes| H[Merge]
    D --> I[Re-run Checks]
    G --> E
    I --> C
    
    style C fill:#f44336
    style F fill:#ff9800
    style H fill:#4caf50
```

### Multi-Level Review
```mermaid
graph LR
    A[Developer] --> B[Senior Developer]
    B --> C[Team Lead]
    C --> D[Manager]
    D --> E{Final Approval}
    E -->|Approved| F[Deploy]
    E -->|Rejected| G[Rework]
    
    style E fill:#ff9800
    style F fill:#4caf50
    style G fill:#f44336
```
