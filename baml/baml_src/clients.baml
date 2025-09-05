
client<llm> CustomGPT4o {
  provider openai
  options {
    model "gpt-4o"
    api_key env.OPENAI_API_KEY
  }
}

client<llm> CustomGPT4oMini {
  provider openai
  retry_policy Exponential
  options {
    model "gpt-4o-mini"
    api_key env.OPENAI_API_KEY
  }
} 

// Client configurations for blockchain agent

// Claude 3 Haiku for fast responses and tool calling
client<llm> CustomHaiku {
  provider anthropic
  retry_policy Exponential
  options {
    model "claude-3-haiku-20240307"
    api_key env.ANTHROPIC_API_KEY
  }
}
// Claude 3 Sonnet for complex reasoning
client<llm> CustomSonnet {
  provider anthropic
  retry_policy Exponential
  options {
    model "claude-3-sonnet-20240229"
    api_key env.ANTHROPIC_API_KEY
  }
}

// GPT-4 for backup and comparison
client<llm> CustomGPT4 {
  provider openai
  retry_policy Exponential
  options {
    model "gpt-4-0125-preview"
    api_key env.OPENAI_API_KEY
  }
}
// Retry policy for exponential backoff
retry_policy Exponential {
  max_retries 3
  strategy {
    type exponential_backoff
    delay_ms 200
    multiplier 2.0
    max_delay_ms 5000
  }
}// Retry policy for constant delay
retry_policy Constant {
  max_retries 2
  strategy {
    type constant_delay
    delay_ms 500
  }
}

// Round-robin strategy for load balancing
client<llm> LoadBalanced {
  provider round-robin
  options {
    strategy [CustomHaiku, CustomGPT4]
  }
}

// Fallback strategy for reliability
client<llm> Fallback {
  provider fallback
  options {
    strategy [CustomHaiku, CustomSonnet, CustomGPT4]
  }
}


