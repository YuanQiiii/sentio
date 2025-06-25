// MongoDB initialization script for Sentio AI
// This script creates the database, collections, and indexes

// Switch to sentio database
db = db.getSiblingDB('sentio');

// Create collections with validation schemas
db.createCollection('user_profiles', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'created_at', 'updated_at'],
            properties: {
                user_id: {
                    bsonType: 'string',
                    description: 'Unique user identifier'
                },
                basic_info: {
                    bsonType: 'object',
                    properties: {
                        name: { bsonType: 'string' },
                        email: { bsonType: 'string' },
                        location: { bsonType: 'string' },
                        occupation: { bsonType: 'string' },
                        age_range: { bsonType: 'string' }
                    }
                },
                relationships: {
                    bsonType: 'array',
                    items: {
                        bsonType: 'object',
                        properties: {
                            contact_id: { bsonType: 'string' },
                            name: { bsonType: 'string' },
                            relationship_type: { bsonType: 'string' },
                            importance_level: { bsonType: 'int' }
                        }
                    }
                },
                personality_traits: {
                    bsonType: 'object',
                    properties: {
                        communication_style: { bsonType: 'string' },
                        formality_preference: { bsonType: 'string' },
                        response_speed_preference: { bsonType: 'string' },
                        interests: { bsonType: 'array' },
                        values: { bsonType: 'array' }
                    }
                },
                created_at: { bsonType: 'date' },
                updated_at: { bsonType: 'date' }
            }
        }
    }
});

db.createCollection('interaction_history', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'interaction_id', 'timestamp', 'interaction_type'],
            properties: {
                user_id: { bsonType: 'string' },
                interaction_id: { bsonType: 'string' },
                timestamp: { bsonType: 'date' },
                interaction_type: {
                    bsonType: 'string',
                    enum: ['email_sent', 'email_received', 'email_replied']
                },
                participants: {
                    bsonType: 'object',
                    properties: {
                        from: { bsonType: 'string' },
                        to: { bsonType: 'array' },
                        cc: { bsonType: 'array' },
                        bcc: { bsonType: 'array' }
                    }
                },
                content_summary: { bsonType: 'string' },
                sentiment_analysis: {
                    bsonType: 'object',
                    properties: {
                        sentiment: { bsonType: 'string' },
                        confidence: { bsonType: 'double' },
                        emotions: { bsonType: 'array' }
                    }
                },
                context_tags: { bsonType: 'array' },
                follow_up_required: { bsonType: 'bool' }
            }
        }
    }
});

db.createCollection('semantic_memory', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'memory_id', 'created_at'],
            properties: {
                user_id: { bsonType: 'string' },
                memory_id: { bsonType: 'string' },
                memory_type: {
                    bsonType: 'string',
                    enum: ['preference', 'habit', 'important_event', 'goal', 'concern']
                },
                content: { bsonType: 'string' },
                importance_score: {
                    bsonType: 'double',
                    minimum: 0.0,
                    maximum: 1.0
                },
                related_interactions: { bsonType: 'array' },
                tags: { bsonType: 'array' },
                created_at: { bsonType: 'date' },
                last_accessed: { bsonType: 'date' },
                access_count: { bsonType: 'int' }
            }
        }
    }
});

db.createCollection('action_memory', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'action_id', 'created_at', 'action_type'],
            properties: {
                user_id: { bsonType: 'string' },
                action_id: { bsonType: 'string' },
                action_type: {
                    bsonType: 'string',
                    enum: ['todo', 'reminder', 'follow_up', 'scheduled_action']
                },
                title: { bsonType: 'string' },
                description: { bsonType: 'string' },
                priority: {
                    bsonType: 'string',
                    enum: ['low', 'medium', 'high', 'urgent']
                },
                status: {
                    bsonType: 'string',
                    enum: ['pending', 'in_progress', 'completed', 'cancelled']
                },
                due_date: { bsonType: 'date' },
                reminder_time: { bsonType: 'date' },
                related_interactions: { bsonType: 'array' },
                tags: { bsonType: 'array' },
                created_at: { bsonType: 'date' },
                updated_at: { bsonType: 'date' },
                completed_at: { bsonType: 'date' }
            }
        }
    }
});

db.createCollection('strategy_memory', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'strategy_id', 'created_at', 'strategy_type'],
            properties: {
                user_id: { bsonType: 'string' },
                strategy_id: { bsonType: 'string' },
                strategy_type: {
                    bsonType: 'string',
                    enum: ['communication', 'decision_making', 'problem_solving', 'learning']
                },
                context: { bsonType: 'string' },
                strategy_content: { bsonType: 'string' },
                effectiveness_score: {
                    bsonType: 'double',
                    minimum: 0.0,
                    maximum: 1.0
                },
                usage_count: { bsonType: 'int' },
                last_used: { bsonType: 'date' },
                success_rate: {
                    bsonType: 'double',
                    minimum: 0.0,
                    maximum: 1.0
                },
                related_interactions: { bsonType: 'array' },
                feedback: { bsonType: 'string' },
                created_at: { bsonType: 'date' },
                updated_at: { bsonType: 'date' }
            }
        }
    }
});

// Create indexes for performance optimization
print('Creating indexes...');

// User profiles indexes
db.user_profiles.createIndex({ user_id: 1 }, { unique: true });
db.user_profiles.createIndex({ 'basic_info.email': 1 });
db.user_profiles.createIndex({ updated_at: -1 });

// Interaction history indexes
db.interaction_history.createIndex({ user_id: 1, timestamp: -1 });
db.interaction_history.createIndex({ interaction_id: 1 }, { unique: true });
db.interaction_history.createIndex({ 'participants.from': 1 });
db.interaction_history.createIndex({ 'participants.to': 1 });
db.interaction_history.createIndex({ interaction_type: 1 });
db.interaction_history.createIndex({ follow_up_required: 1 });

// Semantic memory indexes
db.semantic_memory.createIndex({ user_id: 1, importance_score: -1 });
db.semantic_memory.createIndex({ memory_id: 1 }, { unique: true });
db.semantic_memory.createIndex({ memory_type: 1 });
db.semantic_memory.createIndex({ tags: 1 });
db.semantic_memory.createIndex({ last_accessed: -1 });

// Action memory indexes
db.action_memory.createIndex({ user_id: 1, status: 1 });
db.action_memory.createIndex({ action_id: 1 }, { unique: true });
db.action_memory.createIndex({ action_type: 1 });
db.action_memory.createIndex({ priority: 1 });
db.action_memory.createIndex({ due_date: 1 });
db.action_memory.createIndex({ reminder_time: 1 });

// Strategy memory indexes
db.strategy_memory.createIndex({ user_id: 1, effectiveness_score: -1 });
db.strategy_memory.createIndex({ strategy_id: 1 }, { unique: true });
db.strategy_memory.createIndex({ strategy_type: 1 });
db.strategy_memory.createIndex({ success_rate: -1 });
db.strategy_memory.createIndex({ last_used: -1 });

// Create compound indexes for common query patterns
db.interaction_history.createIndex({ user_id: 1, interaction_type: 1, timestamp: -1 });
db.semantic_memory.createIndex({ user_id: 1, memory_type: 1, importance_score: -1 });
db.action_memory.createIndex({ user_id: 1, action_type: 1, status: 1 });

print('Database initialization completed successfully!');
print('Collections created: user_profiles, interaction_history, semantic_memory, action_memory, strategy_memory');
print('Indexes created for optimal query performance');
